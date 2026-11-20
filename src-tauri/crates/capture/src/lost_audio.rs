// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Audio lost during a recording, written back as silence. A recording's
//! audio is stored as one run of samples, whatever time each buffer was
//! captured at: an AAC track in the fragmented movie keeps its packets back
//! to back, and the raw sidecars on Windows are written straight through. A
//! buffer that never arrives (dropped under load, overrun in the device, or
//! never sent, as a Windows loopback sends nothing while the computer is
//! quiet) therefore leaves no gap, and everything after it plays that much
//! early: over a long recording the voice drifts seconds ahead of the
//! picture. Each track keeps to the clock its buffers are stamped on, and
//! where a buffer arrives further past its track's end than capture jitter
//! accounts for, silence fills the difference first.

use std::time::Instant;

use crate::microphone::{Buffer, Format};

const NANOS_PER_SEC: u64 = 1_000_000_000;

/// How far past its track's end a buffer may be stamped before the
/// difference counts as lost audio: well above callback jitter, and below
/// the 45 ms by which sound leading the picture starts to be noticed.
const LOST_AUDIO_NS: i64 = 40_000_000;

/// The audio lost before a buffer stamped `pts_ns`, on a track that has
/// reached `end_ns`: from where the track ends to that stamp. A track with
/// nothing on it yet has reached its start.
pub(crate) fn lost_before(end_ns: Option<i64>, pts_ns: i64) -> Option<(i64, i64)> {
  let end_ns = end_ns.unwrap_or(0);
  (pts_ns.saturating_sub(end_ns) > LOST_AUDIO_NS).then_some((end_ns, pts_ns))
}

/// Silence from `from_ns` to `to_ns` in `format`, as buffers of at most a
/// second, each with the timestamp it starts at.
pub(crate) fn silence(
  format: Format,
  captured_at: Instant,
  from_ns: i64,
  to_ns: i64,
) -> impl Iterator<Item = (i64, Buffer)> {
  let rate = u64::from(format.sample_rate.max(1));
  let channels = usize::from(format.channels.max(1));
  let span_ns = u64::try_from(to_ns.saturating_sub(from_ns)).unwrap_or_default();
  let total = (span_ns * rate).div_ceil(NANOS_PER_SEC);
  let mut written = 0;
  std::iter::from_fn(move || {
    if written >= total {
      return None;
    }
    let frames = (total - written).min(rate);
    let offset_ns = i64::try_from(written * NANOS_PER_SEC / rate).unwrap_or(i64::MAX);
    let pts_ns = from_ns.saturating_add(offset_ns);
    written += frames;
    let samples = vec![0.0; usize::try_from(frames).unwrap_or_default() * channels];
    Some((
      pts_ns,
      Buffer {
        captured_at,
        samples,
      },
    ))
  })
}
