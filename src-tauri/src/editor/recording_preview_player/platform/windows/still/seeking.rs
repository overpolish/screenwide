// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! When the still decoder seeks, decodes on, reopens, or shows again what it
//! already has.

/// Whether a seek to `position_ms` can present the frames the last settled
/// seek decoded at `settled_ms` again. Only a settled seek can: a rough one
/// is a timeline drag, which moves the readers on.
pub(super) fn reuses_settled(settled_ms: Option<u64>, position_ms: u64, rough: bool) -> bool {
  !rough && settled_ms == Some(position_ms)
}

// Small forward moves are cheaper to satisfy from the decoder's current GPU
// stream. Larger jumps seek directly so rapid timeline scrubs cannot leave a
// growing queue of intermediate frames to decode. The break-even point is one
// group of pictures: our writer keys every half second, so anything inside
// 500 ms costs less decoded forward than a seek's own preroll would. The final
// (non-rough) seek always resets to the exact requested position.
const MAX_SEQUENTIAL_SCRUB_MS: u64 = 500;
const MAX_STILL_LAG_MS: u64 = 100;

pub(super) fn frame_is_stale(frame_ms: u64, position_ms: u64) -> bool {
  frame_ms.saturating_add(MAX_STILL_LAG_MS) < position_ms
}

pub(super) fn should_seek(current_ms: u64, position_ms: u64, rough: bool) -> bool {
  !rough
    || position_ms < current_ms
    || position_ms.saturating_sub(current_ms) > MAX_SEQUENTIAL_SCRUB_MS
}

#[cfg(test)]
mod tests {
  use super::{frame_is_stale, reuses_settled, should_seek};
  use screenwide_export::decode::windows::gpu_decoder::seek_preroll_ms;

  #[test]
  fn rough_scrubs_only_decode_short_forward_steps_sequentially() {
    assert!(!should_seek(4_000, 4_200, true));
    assert!(!should_seek(4_000, 4_500, true));
    assert!(should_seek(4_000, 4_501, true));
    assert!(should_seek(4_000, 3_999, true));
  }

  #[test]
  fn rough_seeks_do_not_preroll() {
    assert_eq!(seek_preroll_ms(true), 0);
    assert_eq!(seek_preroll_ms(false), 1_500);
  }

  #[test]
  fn settled_scrubs_always_seek_exactly() {
    assert!(should_seek(4_000, 4_001, false));
    assert!(should_seek(4_000, 4_000, false));
  }

  #[test]
  fn a_gpu_reader_that_hits_early_eof_is_reopened() {
    assert!(frame_is_stale(17_891, 19_480));
    assert!(!frame_is_stale(19_430, 19_480));
  }

  #[test]
  fn only_a_settled_seek_to_the_same_position_reuses_its_frames() {
    assert!(reuses_settled(Some(4_000), 4_000, false));
    assert!(!reuses_settled(Some(4_000), 4_000, true));
    assert!(!reuses_settled(Some(4_000), 4_001, false));
    assert!(!reuses_settled(None, 4_000, false));
  }
}
