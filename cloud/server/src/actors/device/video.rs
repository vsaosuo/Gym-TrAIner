use std::sync::Arc;

use anyhow::Context;
use common_types::{Feedback, Frame, UserId, VideoId, WorkoutType, IMAGE_HEIGHT, IMAGE_WIDTH};
use firestore::{struct_path::paths, FirestoreDb, FirestoreTimestamp, ParentPathBuilder};
use google_cloud_storage::{
    client::Client as StorageClient,
    http::objects::upload::{Media, UploadObjectRequest, UploadType},
};
use image::{ImageBuffer, RgbImage};
use rayon::prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use rgb565::Rgb565;
use serde::{Deserialize, Serialize};
use tokio::{process::Command, sync::mpsc::UnboundedReceiver};
use uuid::Uuid;

use crate::{actors::device::NAME_WIDTH, constants::*, types::state::AppState};

#[derive(Debug)]
pub(super) enum VideoPart {
    Frames(Vec<Frame>),
    Done,
}

#[tracing::instrument(skip_all, err(Debug))]
pub(super) async fn video_task(
    state: Arc<AppState>,
    mut video_rx: UnboundedReceiver<VideoPart>,
    user_id: UserId,
    workout_type: WorkoutType,
) -> anyhow::Result<()> {
    // create video folder
    let video_id = VideoId::from(Uuid::new_v4().to_string());
    let folder_path: Arc<str> = format!("{VIDEO_PATH}/{video_id}.d").into();

    tracing::debug!("Creating folder: {}", &*folder_path);
    tokio::fs::create_dir(&*folder_path).await?;

    // number of frames received
    let mut count = 0;

    loop {
        match video_rx.recv().await {
            Some(VideoPart::Frames(frames)) => {
                // save all images
                // combine oneshot with rayon
                let (send, recv) = tokio::sync::oneshot::channel();
                let len = frames.len();
                let folder_path = folder_path.clone();

                rayon::spawn(move || {
                    let res = frames.into_par_iter().enumerate().try_for_each(
                        |(i, frame)| -> anyhow::Result<()> {
                            let img =
                                frame_to_image(frame, IMAGE_HEIGHT as u32, IMAGE_WIDTH as u32)?;
                            let filename =
                                format!("{folder_path}/{j:0NAME_WIDTH$}.png", j = count + i);
                            img.save(&filename)
                                .context(format!("Failed to save file: {filename}"))?;

                            Ok(())
                        },
                    );

                    _ = send.send(res);
                });

                // acts like an async join
                recv.await??;

                count += len;
            }
            Some(VideoPart::Done) => {
                handle_video(state, user_id, video_id, &folder_path, workout_type).await?;
                break;
            }
            None => {
                // connection dropped, delete video folder
                tracing::debug!("Deleting folder: {}", &*folder_path);
                tokio::fs::remove_dir_all(&*folder_path).await?;

                // remember to exit the loop!
                break;
            }
        }
    }

    Ok(())
}

async fn handle_video(
    state: Arc<AppState>,
    user_id: UserId,
    video_id: VideoId,
    folder_path: &str,
    workout_type: WorkoutType,
) -> anyhow::Result<()> {
    let date = FirestoreTimestamp::from(chrono::offset::Utc::now());

    let parent_path = state.db.parent_path(USER_COLLECTION, user_id.as_ref())?;

    tracing::debug!("Started processing {video_id:?}");

    let entry = upload_entry(&state.db, &parent_path, date, workout_type).await?;
    tracing::debug!("Uploaded empty entry for {video_id:?}");

    let feedback = call_ml(&video_id, folder_path, workout_type).await?;

    upload_feedback(&state.db, &parent_path, entry, &video_id, feedback).await?;
    tracing::debug!("Uploaded feedback for {video_id:?}");

    let video_path = format!("{VIDEO_PATH}/{video_id}.mp4");
    upload_video(&state.client, &video_id, &video_path).await?;
    tracing::debug!("Uploaded video for {video_id:?}");

    // delete video file and folder
    tracing::debug!("Deleting video {:?} and folder {}", video_id, &folder_path);
    tokio::fs::remove_file(video_path).await?;
    tokio::fs::remove_dir_all(folder_path).await?;

    Ok(())
}

fn frame_to_image(Frame(buf): Frame, height: u32, width: u32) -> anyhow::Result<RgbImage> {
    // need to convert little endian to rgb
    let buf: Vec<_> = buf
        .chunks_exact(2)
        .flat_map(|c| Rgb565::from_rgb565_le([c[0], c[1]]).to_rgb888_components())
        .collect();

    ImageBuffer::from_vec(width, height, buf).context("Failed to create image")
}

async fn call_ml(
    video_id: &VideoId,
    folder_path: &str,
    workout_type: WorkoutType,
) -> anyhow::Result<Vec<Feedback>> {
    let ml_path = match workout_type {
        WorkoutType::Squat => "./.ml/squatPredictor.py",
        WorkoutType::Pushup => "./.ml/pushupPredictor.py",
    };

    let res = Command::new("python")
        .args([
            ml_path,
            &format!("{folder_path}/%0{NAME_WIDTH}d.png"),
            &format!("{VIDEO_PATH}/{video_id}.mp4"),
        ])
        .output()
        .await?;

    tracing::debug!(
        "stdout from {ml_path}: {}",
        String::from_utf8_lossy(&res.stdout)
    );
    tracing::debug!(
        "stderr from {ml_path}: {}",
        String::from_utf8_lossy(&res.stderr)
    );

    Ok(serde_json::from_slice(&res.stdout)?)
}

async fn upload_video(
    client: &StorageClient,
    video_id: &VideoId,
    video_path: &str,
) -> anyhow::Result<()> {
    tracing::debug!("Video path is: {video_path}");
    let video = tokio::fs::read(&video_path).await?;

    let upload_type = UploadType::Simple(Media {
        name: format!("videos/{video_id}").into(),
        content_type: "video/mp4".into(),
        content_length: None,
    });

    // NOTE: using firebase emulators:exec breaks this for some reason!!!
    client
        .upload_object(
            &UploadObjectRequest {
                bucket: BUCKET_NAME.into(),
                ..Default::default()
            },
            video,
            &upload_type,
        )
        .await?;

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct WorkoutEntry {
    #[serde(alias = "_firestore_id")]
    id: Option<String>,
    date: FirestoreTimestamp,
    #[serde(rename = "type")]
    workout_type: WorkoutType,
    video_id: Option<VideoId>,
    reps: Option<Vec<Feedback>>,
}

// uploads the workout entry without video id or feedback
async fn upload_entry(
    db: &FirestoreDb,
    parent_path: &ParentPathBuilder,
    date: FirestoreTimestamp,
    workout_type: WorkoutType,
) -> anyhow::Result<WorkoutEntry> {
    let entry = WorkoutEntry {
        id: None,
        date,
        workout_type,
        video_id: None,
        reps: None,
    };

    let entry = db
        .fluent()
        .insert()
        .into(WORKOUT_COLLECTION)
        .generate_document_id()
        .parent(parent_path)
        .object(&entry)
        .execute::<WorkoutEntry>()
        .await?;

    Ok(entry)
}

async fn upload_feedback(
    db: &FirestoreDb,
    parent_path: &ParentPathBuilder,
    entry: WorkoutEntry,
    video_id: &VideoId,
    feedback: Vec<Feedback>,
) -> anyhow::Result<()> {
    let entry = WorkoutEntry {
        video_id: Some(video_id.clone()),
        reps: Some(feedback),
        ..entry
    };

    db.fluent()
        .update()
        .fields(paths!(WorkoutEntry::{video_id, reps}))
        .in_col(WORKOUT_COLLECTION)
        .document_id(entry.id.as_ref().unwrap())
        .parent(parent_path)
        .object(&entry)
        .execute::<WorkoutEntry>()
        .await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::open_storage;

    use super::*;
    #[tokio::test]
    async fn test_video_upload() -> anyhow::Result<()> {
        let client = open_storage().await;
        let video_id = VideoId::from(Uuid::new_v4().to_string());
        let video_path = "../.video/test.mp4";

        upload_video(&client, &video_id, video_path).await?;

        Ok(())
    }
}
