// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Listening on macOS: the audio-only capture, its microphone and system
//! audio streams as a recording starts them, with a listener in place of the
//! writer. Each run of samples is handed over with how long after listening
//! began it was heard; nothing is written.

use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::thread::JoinHandle;

use super::output::Command;
use super::session::CaptureSession;
use super::{startup, MicrophoneFormat, FRAME_QUEUE_DEPTH};
use super::{SYSTEM_AUDIO_CHANNELS, SYSTEM_AUDIO_SAMPLE_RATE};
use crate::listening::{Heard, HeardSource, Listener, Listening};

/// Listening under way. Stopped, or dropped, it lets go of the microphone
/// and the computer's audio.
pub struct ListeningSession(CaptureSession);

impl ListeningSession {
  pub fn stop(self) {
    self.0.cancel();
  }
}

/// Starts listening as `listening` says, handing what is heard to
/// `listener`.
pub fn begin_listening_blocking(
  listening: Listening,
  listener: Listener,
) -> Result<ListeningSession, String> {
  let start = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .map_err(|error| error.to_string())?
    .block_on(startup::listen(listening, listener))?;
  Ok(ListeningSession(start.session))
}

/// The listener thread's commands, word once listening has begun, and the
/// thread itself.
type Spawned = (
  SyncSender<Command>,
  Receiver<Result<(), String>>,
  JoinHandle<()>,
);

/// The thread that hands what the capture hears to `listener`, in place of
/// a writer.
pub(super) fn spawn(
  listener: Listener,
  microphone: Option<MicrophoneFormat>,
) -> Result<Spawned, String> {
  let (commands, inbox) = mpsc::sync_channel(FRAME_QUEUE_DEPTH);
  let (begun, first_framed) = mpsc::channel();
  let worker = std::thread::Builder::new()
    .name("screenwide-listener".to_owned())
    .spawn(move || listen(&inbox, &listener, microphone, &begun))
    .map_err(|error| error.to_string())?;
  Ok((commands, first_framed, worker))
}

fn listen(
  inbox: &Receiver<Command>,
  listener: &Listener,
  microphone: Option<MicrophoneFormat>,
  begun: &Sender<Result<(), String>>,
) {
  let mut origin = None;
  for command in inbox {
    match command {
      Command::Begin { at } => {
        origin = Some(at);
        let _ = begun.send(Ok(()));
      }
      Command::SystemAudio(sample) => {
        if let Some(origin) = origin {
          listener(Heard {
            source: HeardSource::System,
            samples: sample.samples,
            channels: SYSTEM_AUDIO_CHANNELS as u16,
            sample_rate: SYSTEM_AUDIO_SAMPLE_RATE as u32,
            at: sample.wall.saturating_duration_since(origin),
          });
        }
      }
      Command::Microphone(buffer) => {
        if let (Some(origin), Some(format)) = (origin, microphone) {
          listener(Heard {
            source: HeardSource::Microphone,
            samples: buffer.samples,
            channels: format.channels,
            sample_rate: format.sample_rate,
            at: buffer.captured_at.saturating_duration_since(origin),
          });
        }
      }
      Command::Stop { reply, .. } => {
        let _ = reply.send(Err("Listening keeps no recording".to_owned()));
        return;
      }
      Command::MicrophoneFailed(_) | Command::Cancel => return,
      Command::Replay(request) => request.refuse(),
      Command::Frame(_) | Command::Pause { .. } | Command::Resume { .. } => {}
    }
  }
}
