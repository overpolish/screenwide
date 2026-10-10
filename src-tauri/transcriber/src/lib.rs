// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the app and the transcriber say to each other: one JSON object per
//! line, a [`Request`] on the helper's standard input and a [`Reply`] for it
//! on its standard output. The helper runs requests in the order they come
//! and exits when its input closes, so the app owns how long it lives.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Audio is handed over as a file of 32-bit float samples in native byte
/// order, mono, at this rate: what the engine takes, so the helper needs no
/// decoder of its own and the app's FFmpeg does the converting.
pub const SAMPLE_RATE: u32 = 16_000;

/// The rate a voice is cleaned at: the speech enhancer's own, which keeps the
/// full range a microphone records.
pub const CLEAN_SAMPLE_RATE: u32 = 48_000;

/// The rate music is separated at: the separation model's own. Audio for it
/// is stereo, the two sides interleaved, unlike every other job's.
pub const SEPARATE_SAMPLE_RATE: u32 = 44_100;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
  /// Echoed in the reply.
  pub id: u64,
  #[serde(flatten)]
  pub job: Job,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(
  rename_all = "camelCase",
  rename_all_fields = "camelCase",
  tag = "type"
)]
pub enum Job {
  /// Speech written out as text, answered with [`Reply::Done`].
  Transcribe {
    /// The model file to transcribe with.
    model: PathBuf,
    /// What the transcript names the model as.
    model_id: String,
    /// The samples, as [`SAMPLE_RATE`] describes.
    audio: PathBuf,
    /// A language code such as `en`, or nothing to detect it.
    language: Option<String>,
  },
  /// How likely each moment of the audio is to be speech, answered with
  /// [`Reply::Speech`].
  DetectSpeech {
    /// The voice activity model file.
    model: PathBuf,
    /// The samples, as [`SAMPLE_RATE`] describes.
    audio: PathBuf,
  },
  /// The voice in the audio with everything else taken out, answered with
  /// [`Reply::Cleaned`] once it is written.
  CleanSpeech {
    /// The samples, as [`SAMPLE_RATE`] describes but at
    /// [`CLEAN_SAMPLE_RATE`].
    audio: PathBuf,
    /// Where the cleaned samples go, in the same form, lined up with the
    /// samples in and as long.
    output: PathBuf,
  },
  /// The voice in the audio rebuilt as a studio recording, answered with
  /// [`Reply::Restored`] once it is written.
  RestoreSpeech {
    /// The speech encoder of the restoration model.
    features: PathBuf,
    /// Its vocoder.
    decoder: PathBuf,
    /// The samples, as [`SAMPLE_RATE`] describes.
    audio: PathBuf,
    /// Where the restored samples go, in the same form but at
    /// [`CLEAN_SAMPLE_RATE`], lined up with the samples in.
    output: PathBuf,
    /// The stretches to restore, as `[start, end)` in samples at
    /// [`SAMPLE_RATE`], sorted and apart. Everything else comes out silent.
    stretches: Vec<[u64; 2]>,
  },
  /// The singing in the audio, apart from the music, answered with
  /// [`Reply::Separated`] once it is written.
  SeparateVocals {
    /// The separation model file.
    model: PathBuf,
    /// The samples, as [`SEPARATE_SAMPLE_RATE`] describes.
    audio: PathBuf,
    /// Where the singing goes, in the same form, lined up with the samples
    /// in and as long.
    output: PathBuf,
  },
  /// The person in a camera recording told apart from the room behind them,
  /// answered with [`Reply::Matted`] once the matte is written. The helper
  /// decodes and encodes the video with the app's FFmpeg, so no frame passes
  /// through the protocol.
  MatteCamera {
    /// The video matting model file.
    model: PathBuf,
    /// The FFmpeg program to decode and encode with.
    ffmpeg: PathBuf,
    /// The camera recording.
    camera: PathBuf,
    /// The matte's size in pixels: the camera's, or smaller, as the app
    /// asks; the matte is stretched over the camera when drawn.
    width: u32,
    height: u32,
    /// How long the camera is, in milliseconds, which it is split by to be
    /// matted in parts side by side.
    duration_ms: u64,
    /// About how many frames the camera has, for the progress told.
    frames: u64,
    /// The steady rate the matte's frames are written at, one for each frame
    /// of the camera in turn, whatever the camera's own timing.
    frames_per_second: u32,
    /// Where the matte goes: how much of each pixel is the person, as the
    /// brightness of an H.264 video, which every platform decodes.
    matte: PathBuf,
    /// Where the [`MatteTimings`] go, which line the matte's frames up with
    /// the camera's.
    timings: PathBuf,
  },
  /// Objects clicked in one frame of the camera followed through the whole
  /// recording: forwards to the end, told with [`Reply::Halfway`] once
  /// those masks are written, then backwards to the start, answered with
  /// [`Reply::Tracked`].
  FollowObjects {
    /// The object following model's files, in the catalogue's order.
    model: Vec<PathBuf>,
    ffmpeg: PathBuf,
    camera: PathBuf,
    /// The camera's [`MatteTimings`], which number its frames.
    timings: PathBuf,
    /// The frame clicked, by its place in the timings.
    frame: u64,
    objects: Vec<ClickedObject>,
  },
  /// The matte with objects added to the person or taken away, answered
  /// with [`Reply::Composed`] once it is written.
  ComposeMatte {
    ffmpeg: PathBuf,
    /// The camera recording, whose frames guide the objects' edges.
    camera: PathBuf,
    /// The matte as made, and its size and length in frames.
    matte: PathBuf,
    width: u32,
    height: u32,
    frames: u64,
    frames_per_second: u32,
    picks: Vec<PickedMask>,
    /// Where the matte with the objects in goes, in the matte's form.
    output: PathBuf,
  },
}

/// When each camera frame the matte was made from shows: frame `n` of the
/// matte, at `n / frames_per_second` in its own file, is the matte of the
/// camera frame showing from `frames_ms[n]`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatteTimings {
  pub format: u32,
  pub frames_per_second: u32,
  pub frames_ms: Vec<f64>,
}

impl MatteTimings {
  /// Changes whenever a matte made before would be read wrongly now.
  pub const FORMAT: u32 = 5;
}

/// An object clicked, and where its masks go, each a grey H.264 video with a frame for each camera frame it
/// covers, in turn: from the clicked frame to the end, and from the start
/// to the clicked frame.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClickedObject {
  /// Where it was clicked, 0 to 1 across and down the frame: one point, or
  /// several along a line painted over it.
  pub points: Vec<[f32; 2]>,
  pub after: PathBuf,
  pub before: PathBuf,
}

/// An object's masks as [`Job::FollowObjects`] wrote them: frame `n` is the
/// mask of camera frame `first + n`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PickedMask {
  pub path: PathBuf,
  /// Whether the object is added to the person, or taken away.
  pub keep: bool,
  pub first: u64,
  /// The frame the object was clicked in, where its mask is surest.
  pub clicked: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(
  rename_all = "camelCase",
  rename_all_fields = "camelCase",
  tag = "type"
)]
pub enum Reply {
  Done {
    id: u64,
    transcript: Transcript,
  },
  Speech {
    id: u64,
    speech: Speech,
  },
  /// The voice is written; `gain` lifts it back to its recorded loudness.
  Cleaned {
    id: u64,
    gain: f32,
  },
  /// The restored voice is written.
  Restored {
    id: u64,
  },
  /// The singing is written.
  Separated {
    id: u64,
  },
  /// The camera's matte is written.
  Matted {
    id: u64,
  },
  /// The objects' masks after the clicked frame are written, those before
  /// it to come. Sent before the reply that ends the job.
  Halfway {
    id: u64,
  },
  /// An object's masks are written.
  Tracked {
    id: u64,
  },
  /// The matte with the objects in is written.
  Composed {
    id: u64,
  },
  /// How far a long job has got, 0 to 1. Sent any number of times before the
  /// reply that ends the job, which every other kind is.
  Progress {
    id: u64,
    fraction: f32,
  },
  Failed {
    id: u64,
    message: String,
  },
}

/// The chance that each window of the audio holds speech, 0 to 1, in order.
/// Each window is `window_samples` long at [`SAMPLE_RATE`]; the last may run
/// past the end of the audio.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Speech {
  pub probabilities: Vec<f32>,
  pub window_samples: u32,
}

/// Speech as text, with when each word was said. Times are milliseconds from
/// the start of the audio transcribed.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transcript {
  /// The language it was transcribed as, by code: the one asked for, or the
  /// one detected.
  pub language: String,
  pub model: String,
  /// Every segment's text, joined.
  pub text: String,
  pub segments: Vec<Segment>,
}

/// A stretch of speech the engine decoded in one go: roughly a sentence.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
  pub start_ms: u64,
  pub end_ms: u64,
  pub text: String,
  pub words: Vec<Word>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Word {
  pub start_ms: u64,
  pub end_ms: u64,
  pub text: String,
  /// How sure the engine was of it, 0 to 1: low ones are worth a second
  /// look when the text is edited.
  pub probability: f32,
}
