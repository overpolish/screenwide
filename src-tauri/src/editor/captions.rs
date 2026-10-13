// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The recording's captions as the editor makes them: from the transcript
//! kept beside the project and the edit, set in the output canvas, for the
//! Captions lane, the picture, and the subtitle file an export writes. The
//! captions themselves are `screenwide_captions`.

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use screenwide_captions::{
  build, subtitles, Caption, CaptionFrame, CaptionSource, CaptionTrack, Correction, HeardWord,
  RecordingCaptions, SubtitleFile,
};
use screenwide_compositor::annotation_gpu::{caption_face, caption_measure};
use screenwide_export::recording_model::{AudioTrackKind, RecordingAudioTrack};
use screenwide_speech::transcript::RecordingTranscript;
use screenwide_timeline::TimelineRange;
use screenwide_timeline_edit::{validate, RecordingTimelineEdit, TimelinePlan};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use super::{EditorArtifact, EditorState};

/// The canvas an audio recording's captions are set in, which has no
/// picture of its own: a full HD one, as most players show it.
const AUDIO_FRAME: CaptionFrame = CaptionFrame {
  width: 1_920,
  height: 1_080,
};

/// The canvas captions are set in for an output `width` by `height`: its
/// own, unless it is the stand-in an audio recording has for a picture.
pub(in crate::editor) fn frame_for(width: u32, height: u32) -> CaptionFrame {
  if width < 16 || height < 16 {
    AUDIO_FRAME
  } else {
    CaptionFrame { width, height }
  }
}

/// The transcript kept for the recording's microphone, if it has one and it
/// was transcribed.
fn transcript_of(
  project: &Path,
  audio_tracks: &[RecordingAudioTrack],
) -> Option<RecordingTranscript> {
  let microphone = super::speech::first_of(audio_tracks, AudioTrackKind::Microphone)?;
  super::speech::transcript::read_cache(project.parent().unwrap_or(Path::new(".")), microphone)
}

/// What the edited video plays of a recording `duration_ms` long, in the
/// order it plays: the whole recording where the edit keeps it as it was.
fn played(edit: &RecordingTimelineEdit, duration_ms: u64) -> Vec<TimelineRange> {
  TimelinePlan::from_edit(edit, duration_ms).map_or_else(
    || {
      vec![TimelineRange {
        output_start_us: 0,
        source_start_us: 0,
        source_end_us: duration_ms.saturating_mul(1_000),
        playback_rate: 1.0,
      }]
    },
    |plan| plan.ranges().to_vec(),
  )
}

/// The captions `edit` asks of `transcript`, for a recording `duration_ms`
/// long, set in `frame`. None where the edit asks for none.
fn captions_of(
  transcript: &RecordingTranscript,
  edit: &RecordingTimelineEdit,
  captions: &RecordingCaptions,
  duration_ms: u64,
  frame: CaptionFrame,
) -> Vec<Caption> {
  let words: Vec<HeardWord> = transcript
    .words
    .iter()
    .map(|word| HeardWord {
      start_ms: word.start_ms as f64,
      end_ms: word.end_ms as f64,
      text: word.text.clone(),
    })
    .collect();
  let length = duration_ms as f64;
  let corrections: Vec<Correction> = edit
    .transcript_corrections
    .iter()
    .map(|correction| Correction {
      start_ms: correction.source_start * length,
      end_ms: correction.source_end * length,
      text: correction.text.clone(),
    })
    .collect();
  let ranges = played(edit, duration_ms);
  let face = caption_face(&captions.style.normalized().font);
  // Measured in the face the compositor sets them in, so a line breaks
  // where the drawn type would run too wide.
  let mut measure = caption_measure(&face);
  let source = CaptionSource {
    words: &words,
    language: Some(transcript.language.as_str()).filter(|language| !language.is_empty()),
    corrections: &corrections,
    allowed_words: &edit.transcript_allowed_words,
    ranges: &ranges,
    source_duration_ms: length,
  };
  build(&source, captions, frame, &mut measure)
}

/// The captions last made of the recording open in the editor, shown or
/// not, by artifact: the Captions panel lays its text out from them.
static MADE: Mutex<Option<(u64, Vec<Caption>)>> = Mutex::new(None);

/// Tells the windows the captions were made again, so the Captions panel
/// asks for them; [`recording_captions`] answers.
const MADE_EVENT: &str = "editor://recording-captions";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptionsMade {
  artifact_id: u64,
}

/// Makes the captions `edit` asks of the recording open in the editor, set
/// in a `width` by `height` canvas: the preview draws them from now on while
/// they are on, and the Captions lane shows what is answered. They are made
/// while off too, for the Captions panel's text; none without a transcript.
#[tauri::command]
pub fn make_recording_captions(
  app: AppHandle,
  artifact_id: u64,
  edit: RecordingTimelineEdit,
  width: u32,
  height: u32,
) -> Result<Vec<Caption>, String> {
  validate(&edit)?;
  let (project, audio_tracks, duration_ms) = {
    let state = app.state::<EditorState>();
    let artifact = state
      .recording
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    match artifact.as_ref() {
      Some(EditorArtifact::Recording {
        audio_tracks,
        duration_ms,
        id,
        project,
        ..
      }) if *id == artifact_id => (project.clone(), audio_tracks.clone(), *duration_ms),
      _ => return Err("That recording is no longer available in the editor".to_owned()),
    }
  };
  let made = all_captions(
    (&project, &audio_tracks, duration_ms),
    &edit,
    frame_for(width, height),
    || super::caption_styles::default_captions(&app),
  );
  let track = edit
    .captions
    .as_ref()
    .filter(|captions| captions.enabled)
    .zip(made.as_ref())
    .map(|(recording, captions)| CaptionTrack {
      captions: captions.clone(),
      recording: recording.clone(),
    });
  super::recording_preview_player::show_captions(&app, artifact_id, track);
  let captions = made.unwrap_or_default();
  *MADE.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) =
    Some((artifact_id, captions.clone()));
  let _ = app.emit(MADE_EVENT, CaptionsMade { artifact_id });
  Ok(captions)
}

/// The captions last made of the recording `artifact_id`, shown or not, for
/// the Captions panel to lay its text out from; none until they are made.
#[tauri::command]
pub fn recording_captions(artifact_id: u64) -> Vec<Caption> {
  MADE
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .as_ref()
    .filter(|(id, _)| *id == artifact_id)
    .map(|(_, captions)| captions.clone())
    .unwrap_or_default()
}

/// The captions an export draws into its picture, where `edit` shows them
/// and the recording whose manifest is `project` has a transcript to make
/// them from, set in `frame`.
pub(in crate::editor) fn track_for(
  (project, audio_tracks, duration_ms): (&Path, &[RecordingAudioTrack], u64),
  edit: &RecordingTimelineEdit,
  frame: CaptionFrame,
) -> Option<CaptionTrack> {
  let recording = edit.captions.as_ref().filter(|captions| captions.enabled)?;
  let transcript = transcript_of(project, audio_tracks)?;
  Some(CaptionTrack {
    captions: captions_of(&transcript, edit, recording, duration_ms, frame),
    recording: recording.clone(),
  })
}

/// Every caption `edit` makes of the recording whose manifest is `project`,
/// shown in the picture or not: as its captions are set, or as new ones,
/// `default`, would be where it has none. None without a transcript to make
/// them from.
fn all_captions(
  (project, audio_tracks, duration_ms): (&Path, &[RecordingAudioTrack], u64),
  edit: &RecordingTimelineEdit,
  frame: CaptionFrame,
  default: impl FnOnce() -> RecordingCaptions,
) -> Option<Vec<Caption>> {
  let transcript = transcript_of(project, audio_tracks)?;
  let recording = edit
    .captions
    .as_ref()
    .map_or_else(|| Cow::Owned(default()), Cow::Borrowed);
  Some(captions_of(
    &transcript,
    edit,
    &recording,
    duration_ms,
    frame,
  ))
}

/// A subtitle file of `kind` for `edit` of the recording whose manifest is
/// `project`, its captions broken as [`all_captions`] breaks them. None
/// where the recording has no transcript to make them from: the export form
/// offers no subtitles then.
pub(in crate::editor) fn subtitles_for(
  recording: (&Path, &[RecordingAudioTrack], u64),
  edit: &RecordingTimelineEdit,
  frame: CaptionFrame,
  kind: SubtitleFile,
  default: impl FnOnce() -> RecordingCaptions,
) -> Option<(SubtitleFile, String)> {
  let captions = all_captions(recording, edit, frame, default)?;
  Some((kind, subtitles(&captions, kind)))
}

/// Writes `contents`, a subtitle file of `kind`, beside `video`, named as
/// it is.
pub(in crate::editor) fn write_subtitles(
  video: &Path,
  (kind, contents): (SubtitleFile, String),
) -> Result<PathBuf, String> {
  let path = video.with_extension(kind.extension());
  std::fs::write(&path, contents)
    .map_err(|error| format!("Could not write the subtitles: {error}"))?;
  Ok(path)
}
