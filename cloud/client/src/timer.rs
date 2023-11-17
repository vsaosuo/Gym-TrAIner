use tokio::time::{Duration, Instant};

pub(super) struct FpsTimer {
    fps: u32,
    frame: u32,
    next: Instant,
}

impl FpsTimer {
    pub(super) fn new(fps: u32) -> Self {
        assert!(fps > 0, "fps must be positive");

        Self {
            fps,
            frame: 0,
            next: Instant::now(),
        }
    }

    pub(super) async fn tick(&mut self) {
        let now = Instant::now();

        if now >= self.next {
            self.next = now;
            self.frame = 0;
        } else {
            tokio::time::sleep_until(self.next).await;
        }

        self.next += Duration::from_millis(
            ((self.frame + 1) * 1000 / self.fps - self.frame * 1000 / self.fps) as u64,
        );
        self.frame += 1;
        if self.frame == self.fps {
            self.frame = 0;
        }
    }
}
