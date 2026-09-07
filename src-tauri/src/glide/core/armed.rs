// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Owner {
  Mouse,
  Wheel,
  Trackpad,
}

pub(crate) fn claim(armed: &mut bool, candidate: Owner) -> Option<Owner> {
  if *armed {
    *armed = false;
    Some(candidate)
  } else {
    None
  }
}

pub(crate) fn can_claim_scroll(
  armed: bool,
  phase: i64,
  momentum: i64,
  delta_x: i64,
  delta_y: i64,
) -> bool {
  let allowed = armed && momentum == 0 && (delta_x != 0 || delta_y != 0) && phase & (4 | 8) == 0;
  if armed && !allowed && std::env::var_os("SCREENWIDE_GLIDE_TRACE").is_some() {
    eprintln!(
      "[glide-armed] scroll rejected phase={phase} momentum={momentum} delta=({delta_x},{delta_y})"
    );
  }
  allowed
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn only_the_first_claim_can_take_an_armed_preview() {
    let mut armed = true;
    assert_eq!(claim(&mut armed, Owner::Mouse), Some(Owner::Mouse));
    assert_eq!(claim(&mut armed, Owner::Trackpad), None);
  }

  #[test]
  fn scroll_claim_requires_real_nonterminal_motion() {
    assert!(!can_claim_scroll(true, 4, 0, 0, 0));
    assert!(!can_claim_scroll(true, 0, 1, 2, 0));
    assert!(can_claim_scroll(true, 0, 0, 2, 0));
  }
}
