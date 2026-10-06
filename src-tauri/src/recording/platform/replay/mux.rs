// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Writing a saved clip as a working movie the editor opens like any other.
//!
//! Video is passed through as encoded, retimed so the clip starts at zero.
//! The kept PCM is encoded to AAC here, with the same settings a recording's
//! tracks use, and a track that was selected keeps its place even when the
//! clip happens to hold no audio for it.

use std::path::Path;
use std::time::{Duration, Instant};

use cidre::{arc, av, cf, cm, ns, os};

use super::super::asset_writer_error;
use super::super::media::{
  microphone_audio_settings, microphone_format_description, microphone_sample_buffer, nanos,
  system_audio_settings,
};
use super::super::{NANOS_PER_MS, SYSTEM_AUDIO_CHANNELS, SYSTEM_AUDIO_SAMPLE_RATE};
use super::ClipTake;
use crate::recording::microphone::{Buffer as MicrophoneBuffer, Format as MicrophoneFormat};
use crate::recording::replay::ring::AudioChunk;

/// How long a clip may take to write before it is given up on. Thirty
/// seconds of passthrough video and AAC encoding takes well under one.
const WRITE_TIMEOUT: Duration = Duration::from_secs(30);
const READY_POLL: Duration = Duration::from_millis(1);

pub(super) struct WrittenMovie {
  pub duration_ms: u64,
  pub has_microphone: bool,
  pub has_system_audio: bool,
}

#[link(name = "CoreMedia", kind = "framework")]
unsafe extern "C" {
  fn CMSampleBufferCreateCopyWithNewTiming(
    allocator: Option<&cf::Allocator>,
    original: &cm::SampleBuf,
    timing_count: cm::ItemCount,
    timing: *const cm::SampleTimingInfo,
    copy: *mut Option<arc::R<cm::SampleBuf>>,
  ) -> os::Status;
}

/// `sample` at `pts_ns`. Frames are never reordered, so decode time is the
/// presentation time.
fn retimed(sample: &cm::SampleBuf, pts_ns: i64) -> Result<arc::R<cm::SampleBuf>, String> {
  let timing = cm::SampleTimingInfo {
    duration: cm::Time::invalid(),
    pts: nanos(pts_ns),
    dts: nanos(pts_ns),
  };
  let mut copy = None;
  // SAFETY: one timing entry for a one-sample buffer, and `copy` is a valid
  // out-pointer for the retained result.
  unsafe { CMSampleBufferCreateCopyWithNewTiming(None, sample, 1, &timing, &mut copy) }
    .result()
    .map_err(|error| format!("A replay frame could not be retimed: {error:?}"))?;
  copy.ok_or_else(|| "A replay frame could not be retimed".to_owned())
}

struct AudioTrack {
  chunks: Vec<AudioChunk>,
  description: arc::R<cm::AudioFormatDesc>,
  format: MicrophoneFormat,
  input: arc::R<av::AssetWriterInput>,
}

impl AudioTrack {
  fn new(
    writer: &mut av::AssetWriter,
    format: MicrophoneFormat,
    settings: arc::R<ns::DictionaryMut<ns::String, ns::Id>>,
    mut chunks: Vec<AudioChunk>,
  ) -> Result<Self, String> {
    let mut input = av::AssetWriterInput::with_media_type_and_output_settings(
      av::MediaType::audio(),
      Some(&settings),
    )
    .map_err(|error| error.to_string())?;
    input.set_expects_media_data_in_real_time(false);
    writer
      .add_input(&input)
      .map_err(|error| error.to_string())?;
    // AVAssetWriter leaves out a track that never receives a sample, which
    // would shift every later track's index under the editor. A moment of
    // silence keeps it.
    if chunks.is_empty() {
      chunks.push(AudioChunk {
        pts_ns: 0,
        samples: vec![0.0; 1_024 * usize::from(format.channels)],
      });
    }
    Ok(Self {
      chunks,
      description: microphone_format_description(format)?,
      format,
      input,
    })
  }

  fn end_ns(&self) -> i64 {
    self.chunks.last().map_or(0, |chunk| {
      let frames = chunk.samples.len() / usize::from(self.format.channels.max(1));
      chunk.pts_ns + frames as i64 * 1_000_000_000 / i64::from(self.format.sample_rate.max(1))
    })
  }
}

pub(super) fn write_movie(path: &Path, take: &ClipTake) -> Result<WrittenMovie, String> {
  let location = path
    .to_str()
    .ok_or_else(|| "The clip's location cannot be written as text".to_owned())?;
  let url = ns::Url::with_fs_path_str(location, false);
  let mut writer = av::AssetWriter::with_url_and_file_type(&url, av::FileType::qt())
    .map_err(|error| error.to_string())?;

  let mut video_input = match &take.video {
    Some(video) => {
      let first = video
        .frames
        .first()
        .ok_or_else(|| "The clip has no video".to_owned())?;
      let mut input = av::AssetWriterInput::with_media_type_output_settings_source_format_hint(
        av::MediaType::video(),
        None,
        first.sample.0.format_desc(),
      )
      .map_err(|error| error.to_string())?;
      input.set_expects_media_data_in_real_time(false);
      writer
        .add_input(&input)
        .map_err(|error| error.to_string())?;
      Some(input)
    }
    None => None,
  };

  let mut tracks = Vec::new();
  if let Some(chunks) = &take.system_audio {
    let format = MicrophoneFormat {
      channels: SYSTEM_AUDIO_CHANNELS as u16,
      sample_rate: SYSTEM_AUDIO_SAMPLE_RATE as u32,
    };
    let chunks = rebased(chunks, take.start_ns);
    tracks.push(AudioTrack::new(
      &mut writer,
      format,
      system_audio_settings(),
      chunks,
    )?);
  }
  if let Some((format, chunks)) = &take.microphone {
    let chunks = rebased(chunks, take.start_ns);
    let settings = microphone_audio_settings(*format);
    tracks.push(AudioTrack::new(&mut writer, *format, settings, chunks)?);
  }

  if !writer.start_writing() {
    return Err(asset_writer_error(&writer, "The clip could not be started"));
  }
  writer.start_session_at_src_time(cm::Time::zero());

  let result = append_all(&mut writer, take, video_input.as_mut(), &mut tracks);
  if let Err(error) = result {
    writer.cancel_writing();
    let _ = std::fs::remove_file(path);
    return Err(error);
  }

  let end_ns = tracks
    .iter()
    .map(AudioTrack::end_ns)
    .fold(take.end_ns.saturating_sub(take.start_ns), i64::max);
  writer
    .end_session_at_src_time(nanos(end_ns))
    .map_err(|error| error.to_string())?;
  writer.finish_writing();
  if writer.status() != av::AssetWriterStatus::Completed {
    let error = asset_writer_error(&writer, "The clip could not be saved");
    let _ = std::fs::remove_file(path);
    return Err(error);
  }

  Ok(WrittenMovie {
    duration_ms: u64::try_from(end_ns / NANOS_PER_MS).unwrap_or_default(),
    has_microphone: take.microphone.is_some(),
    has_system_audio: take.system_audio.is_some(),
  })
}

fn rebased(chunks: &[AudioChunk], start_ns: i64) -> Vec<AudioChunk> {
  chunks
    .iter()
    .map(|chunk| AudioChunk {
      pts_ns: chunk.pts_ns.saturating_sub(start_ns).max(0),
      samples: chunk.samples.clone(),
    })
    .collect()
}

/// Appends every sample, feeding whichever inputs are ready the way
/// `requestMediaDataWhenReady` would. Strict presentation order across
/// tracks deadlocks: the AAC encoder holds back audio it has been given until
/// it has more, so a video input can wait on audio that order would never
/// send. Each input is marked finished as soon as it has nothing left.
fn append_all(
  writer: &mut av::AssetWriter,
  take: &ClipTake,
  mut video_input: Option<&mut arc::R<av::AssetWriterInput>>,
  tracks: &mut [AudioTrack],
) -> Result<(), String> {
  let frames = take
    .video
    .as_ref()
    .map_or(&[][..], |video| &video.frames[..]);
  let mut video_at = 0;
  let mut audio_at = vec![0; tracks.len()];
  let deadline = Instant::now() + WRITE_TIMEOUT;
  let appended = |writer: &av::AssetWriter, result: Result<bool, _>| match result {
    Ok(true) => Ok(()),
    Ok(false) => Err(asset_writer_error(writer, "The clip could not be written")),
    Err(error) => Err(format!("A clip sample was rejected: {error}")),
  };

  loop {
    let mut progressed = false;
    if let Some(input) = video_input.as_deref_mut() {
      if video_at < frames.len() && input.is_ready_for_more_media_data() {
        // A follower's first keyframe can land a frame after the chosen start;
        // it is shown from the start rather than leaving the clip a frame of
        // nothing.
        let pts_ns = if video_at == 0 {
          0
        } else {
          frames[video_at].pts_ns.saturating_sub(take.start_ns).max(1)
        };
        let sample = retimed(&frames[video_at].sample.0, pts_ns)?;
        appended(writer, input.append_sample_buf(&sample))?;
        video_at += 1;
        progressed = true;
        if video_at == frames.len() {
          input.mark_as_finished();
        }
      }
    }
    for (track, at) in tracks.iter_mut().zip(audio_at.iter_mut()) {
      if *at < track.chunks.len() && track.input.is_ready_for_more_media_data() {
        let chunk = &track.chunks[*at];
        let buffer = MicrophoneBuffer {
          captured_at: Instant::now(),
          samples: chunk.samples.clone(),
        };
        let sample =
          microphone_sample_buffer(&buffer, track.format, &track.description, chunk.pts_ns)?;
        appended(writer, track.input.append_sample_buf(&sample))?;
        *at += 1;
        progressed = true;
        if *at == track.chunks.len() {
          track.input.mark_as_finished();
        }
      }
    }

    let done = video_at == frames.len()
      && tracks
        .iter()
        .zip(&audio_at)
        .all(|(track, at)| *at == track.chunks.len());
    if done {
      return Ok(());
    }
    if !progressed {
      if Instant::now() >= deadline {
        return Err("Writing the clip took too long".to_owned());
      }
      std::thread::sleep(READY_POLL);
    }
  }
}
