// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Carrying the size of a few encoded stretches across a whole video.
//!
//! A screen recording's keyframes cost hundreds of times what the frames
//! between them do, so how many keyframes a stretch holds decides its size
//! as much as what it shows. Stretches joined for an estimate start with a
//! keyframe each, far more often than the export writes them, so the two
//! kinds of frame are measured apart and counted again at the export's
//! own keyframe spacing.

/// The frames of a raw H.264 stream, keyframes and the rest apart.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct SampleFrames {
  key_bytes: u64,
  keys: u64,
  other_bytes: u64,
  others: u64,
}

impl SampleFrames {
  /// The frames of `stream`, an Annex B H.264 stream with one slice to a
  /// frame, as x264 writes with frame threads. Parameter sets and other
  /// headers count toward the frame they come before.
  pub(crate) fn of(stream: &[u8]) -> Self {
    let mut frames = Self::default();
    let mut pending = 0_u64;
    let starts = nal_starts(stream);
    for (index, &start) in starts.iter().enumerate() {
      let end = starts.get(index + 1).map_or(stream.len(), |next| *next);
      let bytes = (end - start) as u64;
      let header = stream[start..end]
        .iter()
        .skip_while(|byte| **byte == 0)
        .nth(1)
        .copied()
        .unwrap_or_default();
      match header & 0x1f {
        5 => {
          frames.keys += 1;
          frames.key_bytes += pending + bytes;
          pending = 0;
        }
        1 => {
          frames.others += 1;
          frames.other_bytes += pending + bytes;
          pending = 0;
        }
        _ => pending += bytes,
      }
    }
    frames
  }

  /// How many frames were measured.
  pub(crate) fn frames(&self) -> u64 {
    self.keys + self.others
  }

  /// The bytes of a whole video `whole_us` long, whose sampled stretches
  /// ran `sampled_us`, with a keyframe every `keyframe_interval` frames.
  pub(crate) fn whole_bytes(&self, sampled_us: u64, whole_us: u64, keyframe_interval: u64) -> u64 {
    let measured = self.key_bytes + self.other_bytes;
    if self.keys == 0 || self.others == 0 {
      return super::estimate::sampled_to_whole(measured, sampled_us, whole_us);
    }
    let frames = self.frames() as f64 * whole_us as f64 / sampled_us.max(1) as f64;
    let keys = (frames / keyframe_interval.max(1) as f64)
      .ceil()
      .min(frames);
    let key = self.key_bytes as f64 / self.keys as f64;
    let other = self.other_bytes as f64 / self.others as f64;
    (keys * key + (frames - keys) * other).round() as u64
  }
}

/// Where each NAL unit starts in `stream`, at its start code.
fn nal_starts(stream: &[u8]) -> Vec<usize> {
  let mut starts = Vec::new();
  let mut index = 0;
  while index + 3 <= stream.len() {
    if stream[index] == 0 && stream[index + 1] == 0 && stream[index + 2] == 1 {
      // A four-byte start code's leading zero belongs to it.
      let start = if index > 0 && stream[index - 1] == 0 {
        index - 1
      } else {
        index
      };
      starts.push(start);
      index += 3;
    } else {
      index += 1;
    }
  }
  starts
}

/// Comma-separated times, in seconds, at which each stretch after the first
/// starts, as `-force_key_frames` takes them: a joined stretch opens on a
/// keyframe, which `SampleFrames` then counts as one, rather than on a
/// frame that costs as much while passing for the rest.
pub(crate) fn stretch_starts(lengths_us: impl IntoIterator<Item = u64>) -> Option<String> {
  let mut at = 0_u64;
  let mut times = Vec::new();
  for length in lengths_us {
    if at > 0 {
      times.push(format!("{:.6}", at as f64 / 1_000_000.0));
    }
    at += length;
  }
  (!times.is_empty()).then(|| times.join(","))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn nal(kind: u8, bytes: usize) -> Vec<u8> {
    let mut unit = vec![0, 0, 0, 1, kind];
    unit.resize(4 + bytes, 0xAA);
    unit
  }

  #[test]
  fn headers_count_toward_the_keyframe_they_open() {
    let stream = [nal(7, 10), nal(8, 5), nal(5, 1_000), nal(1, 20), nal(1, 30)].concat();
    let frames = SampleFrames::of(&stream);
    assert_eq!(
      frames,
      SampleFrames {
        key_bytes: 14 + 9 + 1_004,
        keys: 1,
        other_bytes: 24 + 34,
        others: 2,
      }
    );
  }

  #[test]
  fn whole_video_pays_keyframes_at_its_own_spacing() {
    // Three stretches of 30 frames, each opening on a 1,000 byte keyframe,
    // the rest 10 bytes: a 300 frame video with a keyframe every 150 frames
    // holds two keyframes, not the ten the stretches' rate would give.
    let frames = SampleFrames {
      key_bytes: 3_000,
      keys: 3,
      other_bytes: 870,
      others: 87,
    };
    assert_eq!(
      frames.whole_bytes(1_500_000, 5_000_000, 150),
      2 * 1_000 + 298 * 10
    );
  }

  #[test]
  fn stretches_after_the_first_open_on_a_keyframe() {
    assert_eq!(
      stretch_starts([1_500_000, 1_500_000, 1_000_000]),
      Some("1.500000,3.000000".to_owned())
    );
    assert_eq!(stretch_starts([9_000_000]), None);
  }
}
