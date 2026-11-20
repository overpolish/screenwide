// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Audio's own way to a writer. The writer's channel is shallow on purpose
//! (`FRAME_QUEUE_DEPTH`): each frame waiting in it holds a capture surface,
//! and a frame that finds it full is simply skipped, there being another
//! along at once. Audio that found it full would be a hole in the sound, so
//! audio goes into a deep queue of its own, and a thread passes it on to the
//! writer, waiting for room for as long as the writer is busy.

use super::*;

/// About ten seconds of microphone buffers, and longer of system audio.
const AUDIO_QUEUE_DEPTH: usize = 1024;

/// A sender for audio bound for `commands`. The thread passing it on ends
/// once every such sender is gone, or the writer is.
pub(super) fn audio_queue(
  commands: SyncSender<Command>,
  name: &str,
) -> Result<SyncSender<Command>, String> {
  let (audio, queued) = mpsc::sync_channel(AUDIO_QUEUE_DEPTH);
  std::thread::Builder::new()
    .name(name.to_owned())
    .spawn(move || {
      for command in queued {
        if commands.send(command).is_err() {
          return;
        }
      }
    })
    .map_err(|error| error.to_string())?;
  Ok(audio)
}

#[cfg(test)]
mod tests {
  use super::*;

  /// A writer busy for longer than its channel holds, as a recording's
  /// writer is under load: every buffer still reaches it, in order.
  #[test]
  fn audio_waits_for_a_busy_writer_instead_of_being_dropped() {
    let (commands, inbox) = mpsc::sync_channel(FRAME_QUEUE_DEPTH);
    let audio = audio_queue(commands, "screenwide-test-audio-queue").unwrap();
    let sent = 200;
    for index in 0..sent {
      let buffer = MicrophoneBuffer {
        captured_at: Instant::now(),
        samples: vec![index as f32],
      };
      assert!(audio.try_send(Command::Microphone(buffer)).is_ok());
    }
    drop(audio);
    let received: Vec<f32> = inbox
      .iter()
      .filter_map(|command| match command {
        Command::Microphone(buffer) => Some(buffer.samples[0]),
        _ => None,
      })
      .collect();
    let expected: Vec<f32> = (0..sent).map(|index| index as f32).collect();
    assert_eq!(received, expected);
  }
}
