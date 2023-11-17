mod timer;

use std::{convert::Infallible, num::NonZeroUsize, time::Duration};

use anyhow::{bail, Context};
use clap::Parser;
use common_types::{DeviceResponse, UserId, VideoRequest, WorkoutType, IMAGE_HEIGHT, IMAGE_WIDTH};
use futures::{
    stream::{SplitSink, SplitStream},
    Future, SinkExt, StreamExt,
};
use rgb565::Rgb565;
use tokio::{
    net::TcpStream,
    pin, select,
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    time::{Instant, Interval, MissedTickBehavior},
};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::timer::FpsTimer;
use drivers::{
    Camera, DevMem, HexDisplay, Keys as RawKeys, KeysPressed, Texture, TouchArea, TouchScreen,
    VgaDisplay, TOUCHSCREEN_HEIGHT, TOUCHSCREEN_WIDTH,
};

const KEYS_RATE: Duration = Duration::from_millis(10);
const CAMERA_FPS: u32 = 30;
const DEVICE_ID: &str = "38469b2b-58db-40db-9bb3-e833eb043b30";

// change as needed
const QR_TEXTURE_PATH: &str = "Scan QR Code.png";
const PICK_TEXTURE_PATH: &str = "Pick A Workout.png";
const COUNTDOWN_TEXTURE_PATH: &str = "Countdown.png";
const START_TEXTURE_PATH: &str = "Start Button.png";

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_VIDEO_LENGTH: Duration = Duration::from_secs(5 * 60); // 5 minutes

type WsReadHalf = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
type WsWriteHalf = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

struct Keys {
    keys: RawKeys,
    interval: Interval,
}

// a thin wrapper that throttles the read rate
impl Keys {
    fn new(keys: RawKeys, poll_rate: Duration) -> Self {
        let mut interval = tokio::time::interval(poll_rate);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        Self { keys, interval }
    }

    async fn read(&mut self) -> KeysPressed {
        self.interval.tick().await;
        self.keys.read()
    }
}

struct Peripherals {
    keys: Keys,
    camera: Camera,
    vga: VgaDisplay,
    hex: HexDisplay,
    touch: TouchScreen,
}

struct Resources {
    qr_texture: Texture,
    pick_texture: Texture,
    countdown_texture: Texture,
    start_texture: Texture,
    batch_size: usize,
}

#[derive(Parser)]
struct Args {
    #[arg(long)]
    server_url: String,
    #[arg(long, default_value_t = NonZeroUsize::new(30).unwrap())]
    batch_size: NonZeroUsize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Args {
        server_url,
        batch_size,
    } = Args::parse();

    let ws_url = format!("{server_url}/device?id={DEVICE_ID}");
    println!("Trying to connect to {ws_url}");

    // TODO: do we need any handling in case the server does not stay up? e.g., retry loop?
    let (ws, _) = tokio_tungstenite::connect_async(ws_url)
        .await
        .context("Failed to create websocket")?;
    println!("Websocket connected");

    // NOTE: file does not need to be kept open after memory mapping!

    let mem = DevMem::new().await?;

    let perif = Peripherals {
        keys: Keys::new(RawKeys::new(&mem)?, KEYS_RATE),
        camera: Camera::new(&mem)?,
        vga: VgaDisplay::new(&mem)?,
        hex: HexDisplay::new(&mem)?,
        touch: TouchScreen::new(&mem)?,
    };
    println!("Opened peripherals");

    // we should have: qr code, workout selection, start workout
    let resources = Resources {
        qr_texture: load_texture(QR_TEXTURE_PATH).await?,
        pick_texture: load_texture(PICK_TEXTURE_PATH).await?,
        countdown_texture: load_texture(COUNTDOWN_TEXTURE_PATH).await?,
        start_texture: load_texture(START_TEXTURE_PATH).await?,
        batch_size: batch_size.into(),
    };
    println!("Loaded resources");

    let (req_tx, req_rx) = mpsc::unbounded_channel();
    let (res_tx, res_rx) = mpsc::unbounded_channel();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let (ws_tx, ws_rx) = ws.split();

    spawn_logged(ws_send_loop(req_rx, ws_tx));
    spawn_logged(ws_recv_loop(ws_rx, res_tx, shutdown_tx));

    select! {
        res = connection_loop(res_rx, req_tx, perif, resources) => {
            res?;
        }
        _ = shutdown_rx => {}
    }

    println!("Exiting main");

    // Trying to close the websocket will time out if the connection is already closed
    // so we can just drop the connection

    Ok(())
}

async fn load_texture(path: &str) -> anyhow::Result<Texture> {
    let img = image::open(path)?;
    let img = img.as_rgba8().context("Image was not RGBA8")?;
    let data: Vec<_> = img
        .pixels()
        .map(|p| {
            let [r, g, b, _] = p.0;
            Rgb565::from_rgb888_components(r, g, b).to_rgb565()
        })
        .collect();

    Ok(Texture::new(IMAGE_WIDTH, IMAGE_HEIGHT, data))
}

fn spawn_logged<T>(fut: impl Future<Output = anyhow::Result<T>> + Send + 'static) {
    tokio::spawn(async move {
        if let Err(e) = fut.await {
            eprintln!("{e:?}");
        }
    });
}

async fn connection_loop(
    mut res_rx: UnboundedReceiver<DeviceResponse>,
    mut req_tx: UnboundedSender<VideoRequest>,
    mut perif: Peripherals,
    resources: Resources,
) -> anyhow::Result<()> {
    loop {
        handle_connection(&mut res_rx, &mut req_tx, &mut perif, &resources).await?;
    }
}

// just for forwarding all received messages
// other end can do whatever they want
async fn ws_recv_loop(
    mut ws_rx: WsReadHalf,
    res_tx: UnboundedSender<DeviceResponse>,
    _shutdown_tx: oneshot::Sender<Infallible>, // gets dropped if connection to server is cut
) -> anyhow::Result<()> {
    println!("Spawned ws_recv_loop");

    loop {
        let msg = tokio::time::timeout(CONNECTION_TIMEOUT, ws_rx.next())
            .await
            .context("Server connection timed out")?
            .context("No response from server")??;

        let res: DeviceResponse = match msg {
            Message::Text(msg) => serde_json::from_str(&msg)?,
            Message::Pong(_) => {
                println!("Received pong");
                continue;
            }
            Message::Close(_) => {
                bail!("Server closed connection")
            }
            _ => {
                bail!("Unexpected message from server: {msg:?}")
            }
        };

        res_tx.send(res)?;
    }
}

async fn ws_send_loop(
    mut req_rx: UnboundedReceiver<VideoRequest>,
    mut ws_tx: WsWriteHalf,
) -> anyhow::Result<()> {
    println!("Spawned ws_send_loop");

    const PING_INTERVAL: Duration = Duration::from_secs(5);
    let mut ping = tokio::time::interval(PING_INTERVAL);

    // in case sending video takes too long, we don't care about "catching up"
    ping.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        select! {
            _ = ping.tick() => {
                println!("Sending ping");
                ws_tx.send(Message::Ping(vec![])).await?;
            }
            req = req_rx.recv() => {
                let Some(req) = req else {
                    break;
                };

                let start = Instant::now();
                let msg = Message::Binary(bincode::serialize(&req)?);

                ws_tx.send(msg).await?;

                let end = Instant::now();
                println!("Sending data took: {} ms", (end - start).as_millis());
            }
            _ = tokio::signal::ctrl_c() => {
                // try to close
                ws_tx.close().await?;
                break;
            }
        }
    }

    Ok(())
}

async fn handle_connection(
    res_rx: &mut UnboundedReceiver<DeviceResponse>,
    req_tx: &mut UnboundedSender<VideoRequest>,
    perif: &mut Peripherals,
    resources: &Resources,
) -> anyhow::Result<()> {
    show_qr_code(&mut perif.vga, resources).await;
    let user_id = wait_connection(res_rx).await?;

    // main part of workout
    select! {
        res = wait_disconnection(res_rx) => {
            // TODO: add another screen here?

            // NOTE: in really bad circumstances, there could potentially be more than one video in the outgoing queue
            // if a video gets cut off part way, we don't want to flush the entire queue!
            // so we will simply not using flushing altogether

            // instead, just signal that the last video, if it wasn't done, should be cancelled
            req_tx.send(VideoRequest::Cancel)?;

            res?;
        }
        res = do_workout(req_tx, perif, user_id, resources) => {
            res?;
        }
    }

    Ok(())
}

async fn show_qr_code(vga: &mut VgaDisplay, resources: &Resources) {
    vga.draw_texture(0, 0, &resources.qr_texture);
    vga.sync_screen().await;
}

async fn wait_connection(ws_rx: &mut UnboundedReceiver<DeviceResponse>) -> anyhow::Result<UserId> {
    loop {
        if let DeviceResponse::Connected { user_id } = ws_rx.recv().await.context("ws_rx closed")? {
            println!("Connected to user id: {user_id}");
            return Ok(user_id);
        }
    }
}

async fn wait_disconnection(ws_rx: &mut UnboundedReceiver<DeviceResponse>) -> anyhow::Result<()> {
    loop {
        if let DeviceResponse::Disconnected = ws_rx.recv().await.context("ws_rx closed")? {
            println!("Disconnected from user");
            return Ok(());
        }
    }
}

// be careful not to block for too long in here
async fn do_workout(
    req_tx: &mut UnboundedSender<VideoRequest>,
    perif: &mut Peripherals,
    user_id: UserId,
    resources: &Resources,
) -> anyhow::Result<()> {
    loop {
        let Peripherals {
            keys, vga, touch, ..
        } = perif;

        let workout_type = select_workout(keys, vga, touch, resources).await?;
        println!("Selected workout: {workout_type:?}");
        start_workout(keys, vga, touch, resources).await;
        println!("Starting workout");
        record_workout(req_tx, perif, workout_type, &user_id, resources).await?;
        println!("Stopped workout");
    }
}

// convert from vga coordinates to touch coordinates
const fn vga_area((x1, y1): (usize, usize), (x2, y2): (usize, usize)) -> TouchArea {
    TouchArea::new(
        (
            x1 * TOUCHSCREEN_WIDTH / IMAGE_WIDTH,
            y1 * TOUCHSCREEN_HEIGHT / IMAGE_HEIGHT,
        ),
        (
            x2 * TOUCHSCREEN_WIDTH / IMAGE_WIDTH,
            y2 * TOUCHSCREEN_HEIGHT / IMAGE_HEIGHT,
        ),
    )
}

async fn select_workout(
    keys: &mut Keys,
    vga: &mut VgaDisplay,
    touch: &mut TouchScreen,
    resources: &Resources,
) -> anyhow::Result<WorkoutType> {
    const WORKOUT_AREAS: [TouchArea; 2] = [
        vga_area((11, 8), (106, 103)),
        vga_area((213, 8), (308, 103)),
        // vga_area((11, 125), (106, 220)),
        // vga_area((213, 125), (308, 220)),
    ];

    vga.draw_texture(0, 0, &resources.pick_texture);
    vga.sync_screen().await;

    let workout_type = select! {
        workout_type = async move {
            loop {
                let pressed = keys.read().await;
                if pressed[0] {
                    return WorkoutType::Squat;
                } else if pressed[1] {
                    return WorkoutType::Pushup;
                }
            }
        } => workout_type,
        i = touch.wait_touch(&WORKOUT_AREAS) => {
            if i == 0 {
                WorkoutType::Squat
            } else {
                WorkoutType::Pushup
            }
        }
    };

    Ok(workout_type)
}

async fn start_workout(
    keys: &mut Keys,
    vga: &mut VgaDisplay,
    touch: &mut TouchScreen,
    resources: &Resources,
) {
    const START_AREA: TouchArea = vga_area((101, 81), (217, 197));
    vga.draw_texture(0, 0, &resources.start_texture);
    vga.sync_screen().await;

    // TODO: remove keys later?

    select! {
        _ = async move {
            loop {
                let pressed = keys.read().await;
                if pressed[0] {
                    return;
                }
            }
        } => {}
        _ = touch.wait_touch(&[START_AREA]) => {}
    }
}

async fn record_workout(
    req_tx: &mut UnboundedSender<VideoRequest>,
    perif: &mut Peripherals,
    workout_type: WorkoutType,
    user_id: &UserId,
    resources: &Resources,
) -> anyhow::Result<()> {
    // for now, just the whole display
    const STOP_AREA: TouchArea = vga_area((0, 0), (IMAGE_WIDTH - 1, IMAGE_HEIGHT - 1));

    let Peripherals {
        keys,
        camera,
        vga,
        hex,
        touch,
    } = perif;

    vga.draw_texture(0, 0, &resources.countdown_texture);
    vga.sync_screen().await;

    let mut countdown = tokio::time::interval(Duration::from_secs(1));

    // countdown for ten seconds
    countdown.tick().await;
    hex.write([
        0,
        0,
        0,
        0,
        HexDisplay::digit_to_hex(1),
        HexDisplay::digit_to_hex(0),
    ]);

    for i in (5..=9).rev() {
        countdown.tick().await;

        hex.write([0, 0, 0, 0, 0, HexDisplay::digit_to_hex(i)]);
    }

    // switch to camera, which writes to the front buffer
    if !vga.is_front() {
        vga.sync_screen().await;
    }

    // enable camera
    let guard = camera.enable();

    for i in (1..=4).rev() {
        countdown.tick().await;

        hex.write([0, 0, 0, 0, 0, HexDisplay::digit_to_hex(i)]);
    }

    // one more to show the 1
    countdown.tick().await;

    // add a small sleep to reduce chance of tearing the first frame
    tokio::time::sleep(Duration::from_millis(20)).await;

    let mut fps = FpsTimer::new(CAMERA_FPS);
    let timeout = tokio::time::sleep(MAX_VIDEO_LENGTH);
    let mut frames = vec![];

    let mut hex_timer = tokio::time::interval(Duration::from_secs(1));
    let mut min = 0;
    let mut sec = 0;

    pin!(timeout);

    // now, every video request also needs to contain the session id

    req_tx.send(VideoRequest::Start {
        user_id: user_id.clone(),
        workout_type,
    })?;

    // wait until key 0, next frame, or maximum video length
    select! {
        // one for loop
        res = async {
            loop {
                fps.tick().await;

                let frame = guard.capture_frame();
                frames.push(frame);

                if frames.len() == resources.batch_size {
                    req_tx.send(
                        VideoRequest::Frames(std::mem::take(&mut frames))
                    )?;
                }
            }

            #[allow(unreachable_code)]
            Ok::<(), anyhow::Error>(())
        } => { res?; }
        _ = async {
            loop {
                hex_timer.tick().await;

                display_time(hex, min, sec);

                // update time
                sec += 1;
                if sec == 60 {
                    sec = 0;
                    min += 1;
                }
            }
        } => {}
        // others don't loop
        _ = &mut timeout => {}
        _ = touch.wait_touch(&[STOP_AREA]) => {}
        _ = wait_key_0(keys) => {}
    }

    // clear timer
    hex.clear();

    if !frames.is_empty() {
        req_tx.send(VideoRequest::Frames(frames))?;
    }

    req_tx.send(VideoRequest::Done)?;

    Ok(())
}

fn display_time(hex: &mut HexDisplay, min: u8, sec: u8) {
    let m0 = HexDisplay::digit_to_hex(min);
    let s1 = HexDisplay::digit_to_hex(sec / 10);
    let s0 = HexDisplay::digit_to_hex(sec % 10);
    hex.write([0, 0, 0, m0, s1, s0]);
}

async fn wait_key_0(keys: &mut Keys) {
    loop {
        let pressed = keys.read().await;
        if pressed[0] {
            break;
        }
    }
}
