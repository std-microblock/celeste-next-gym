use std::fs;
use std::path::{Path, PathBuf};

use ffmpeg::{codec, encoder, format, media};
use ffmpeg_next as ffmpeg;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct FinalizePlan {
    pub clips: Vec<FinalizeClip>,
    pub output_path: String,
}

#[derive(Debug, Deserialize)]
pub struct FinalizeClip {
    pub source: String,
    pub duration_seconds: f64,
}

#[derive(Debug, Error)]
pub enum FinalizeError {
    #[error("finalize plan has no clips")]
    Empty,
    #[error("invalid clip duration {0}")]
    Duration(f64),
    #[error("output path has no parent: {0}")]
    MissingParent(PathBuf),
    #[error("cannot create output directory {path}: {source}")]
    CreateDirectory {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot open input {path}: {source}")]
    OpenInput {
        path: PathBuf,
        source: ffmpeg::Error,
    },
    #[error("input has no video stream: {0}")]
    MissingVideo(PathBuf),
    #[error("cannot create output {path}: {source}")]
    CreateOutput {
        path: PathBuf,
        source: ffmpeg::Error,
    },
    #[error("cannot add output stream: {0}")]
    AddStream(ffmpeg::Error),
    #[error("cannot write output header: {0}")]
    Header(ffmpeg::Error),
    #[error("cannot write output packet: {0}")]
    Packet(ffmpeg::Error),
    #[error("cannot write output trailer: {0}")]
    Trailer(ffmpeg::Error),
    #[error("cannot replace final output {path}: {source}")]
    Replace {
        path: PathBuf,
        source: std::io::Error,
    },
}

pub fn finalize(plan: &FinalizePlan) -> Result<(), FinalizeError> {
    if plan.clips.is_empty() {
        return Err(FinalizeError::Empty);
    }
    for clip in &plan.clips {
        if !clip.duration_seconds.is_finite() || clip.duration_seconds <= 0.0 {
            return Err(FinalizeError::Duration(clip.duration_seconds));
        }
    }
    ffmpeg::init().map_err(|source| FinalizeError::OpenInput {
        path: PathBuf::from("FFmpeg initialization"),
        source,
    })?;

    let output_path = Path::new(&plan.output_path);
    let parent = output_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| FinalizeError::MissingParent(output_path.to_owned()))?;
    fs::create_dir_all(parent).map_err(|source| FinalizeError::CreateDirectory {
        path: parent.to_owned(),
        source,
    })?;
    let temporary = temporary_output_path(output_path);
    let _ = fs::remove_file(&temporary);

    let first_path = Path::new(&plan.clips[0].source);
    let first_input = format::input(first_path).map_err(|source| FinalizeError::OpenInput {
        path: first_path.to_owned(),
        source,
    })?;
    let first_video = first_input
        .streams()
        .best(media::Type::Video)
        .ok_or_else(|| FinalizeError::MissingVideo(first_path.to_owned()))?;
    let first_time_base = first_video.time_base();

    let mut output = format::output(&temporary).map_err(|source| FinalizeError::CreateOutput {
        path: temporary.clone(),
        source,
    })?;
    let output_index;
    {
        let mut stream = output
            .add_stream(encoder::find(codec::Id::None))
            .map_err(FinalizeError::AddStream)?;
        stream.set_parameters(first_video.parameters());
        stream.set_time_base(first_time_base);
        // Avoid carrying a container-specific codec tag from Matroska into MP4.
        unsafe {
            (*stream.parameters().as_mut_ptr()).codec_tag = 0;
        }
        output_index = stream.index();
    }
    drop(first_input);
    output.write_header().map_err(FinalizeError::Header)?;
    let output_time_base = output
        .stream(output_index)
        .expect("newly-added output stream disappeared")
        .time_base();
    let mut output_offset = 0_i64;

    for clip in &plan.clips {
        let source_path = Path::new(&clip.source);
        let mut input = format::input(source_path).map_err(|source| FinalizeError::OpenInput {
            path: source_path.to_owned(),
            source,
        })?;
        let video = input
            .streams()
            .best(media::Type::Video)
            .ok_or_else(|| FinalizeError::MissingVideo(source_path.to_owned()))?;
        let input_index = video.index();
        let input_time_base = video.time_base();
        let limit = seconds_to_ticks(clip.duration_seconds, input_time_base);
        let mut first_timestamp = None;

        for (stream, mut packet) in input.packets() {
            if stream.index() != input_index {
                continue;
            }
            let timestamp = packet.dts().or(packet.pts()).unwrap_or(0);
            let base = *first_timestamp.get_or_insert(timestamp);
            if timestamp.saturating_sub(base) >= limit {
                break;
            }
            packet.set_pts(packet.pts().map(|value| value.saturating_sub(base)));
            packet.set_dts(packet.dts().map(|value| value.saturating_sub(base)));
            packet.rescale_ts(input_time_base, output_time_base);
            packet.set_pts(
                packet
                    .pts()
                    .map(|value| value.saturating_add(output_offset)),
            );
            packet.set_dts(
                packet
                    .dts()
                    .map(|value| value.saturating_add(output_offset)),
            );
            packet.set_stream(output_index);
            packet.set_position(-1);
            packet
                .write_interleaved(&mut output)
                .map_err(FinalizeError::Packet)?;
        }
        output_offset =
            output_offset.saturating_add(seconds_to_ticks(clip.duration_seconds, output_time_base));
    }

    output.write_trailer().map_err(FinalizeError::Trailer)?;
    drop(output);
    fs::remove_file(output_path).ok();
    fs::rename(&temporary, output_path).map_err(|source| FinalizeError::Replace {
        path: output_path.to_owned(),
        source,
    })?;
    Ok(())
}

fn seconds_to_ticks(seconds: f64, time_base: ffmpeg::Rational) -> i64 {
    let ticks = seconds * time_base.denominator() as f64 / time_base.numerator() as f64;
    ticks.round().clamp(1.0, i64::MAX as f64) as i64
}

fn temporary_output_path(output: &Path) -> PathBuf {
    let extension = output
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("mkv");
    output.with_extension(format!("working.{extension}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::VideoFileEncoder;
    use crate::{CaptureConfig, CapturedFrame};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn remuxes_multiple_native_segments_when_enabled() {
        if std::env::var_os("MQOL_TEST_FFMPEG").is_none() {
            return;
        }
        let directory = tempfile::tempdir().unwrap();
        let first = directory.path().join("first.mkv");
        let second = directory.path().join("second.mkv");
        write_segment(&first, 20);
        write_segment(&second, 180);
        let output = directory.path().join("joined.mp4");
        finalize(&FinalizePlan {
            clips: vec![
                FinalizeClip {
                    source: first.to_string_lossy().into_owned(),
                    duration_seconds: 0.5,
                },
                FinalizeClip {
                    source: second.to_string_lossy().into_owned(),
                    duration_seconds: 0.5,
                },
            ],
            output_path: output.to_string_lossy().into_owned(),
        })
        .unwrap();
        assert!(fs::metadata(output).unwrap().len() > 1_000);
        let joined_path = directory.path().join("joined.mp4");
        let mut joined = format::input(&joined_path).unwrap();
        let stream = joined.streams().best(media::Type::Video).unwrap();
        let stream_index = stream.index();
        let time_base = stream.time_base();
        let mut last_timestamp = 0_i64;
        for (stream, packet) in joined.packets() {
            if stream.index() == stream_index {
                last_timestamp = last_timestamp.max(packet.pts().unwrap_or(0));
            }
        }
        assert!(last_timestamp as f64 * f64::from(time_base) > 0.8);
    }

    fn write_segment(path: &Path, red: u8) {
        let config = CaptureConfig {
            output_path: Some(path.to_string_lossy().into_owned()),
            encoder: "libopenh264".to_owned(),
            fps: 30,
            bitrate_kbps: 1_000,
            ..CaptureConfig::default()
        };
        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let mut frame = CapturedFrame {
            width: 64,
            height: 64,
            captured_at_unix_nanos: start,
            bgra: vec![0; 64 * 64 * 4],
        };
        let mut encoder = VideoFileEncoder::create(&config, &frame).unwrap();
        for index in 0..30_u64 {
            frame.captured_at_unix_nanos = start + index * 1_000_000_000 / 30;
            for pixel in frame.bgra.chunks_exact_mut(4) {
                pixel[2] = red;
                pixel[3] = 255;
            }
            encoder.encode(&frame).unwrap();
        }
        encoder.finish().unwrap();
    }
}
