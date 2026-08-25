// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Scalar animation curves shared by rendered export effects.

pub(crate) fn ease_out_cubic(value: f32) -> f32 {
  1.0 - (1.0 - value).powi(3)
}

pub(crate) fn damped_spring(value: f32, decay: f32, angular_frequency: f32) -> f32 {
  if value >= 1.0 {
    1.0
  } else {
    let phase = angular_frequency * value;
    1.0 - (-decay * value).exp() * (phase.cos() + (decay / angular_frequency) * phase.sin())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn previous_ease_out(value: f32) -> f32 {
    1.0 - (1.0 - value).powi(3)
  }

  fn previous_pop_spring(value: f32) -> f32 {
    if value >= 1.0 {
      1.0
    } else {
      let phase = 6.0 * value;
      1.0 - (-5.0 * value).exp() * (phase.cos() + (5.0 / 6.0) * phase.sin())
    }
  }

  #[test]
  fn extracted_curves_are_bit_identical_to_the_keyboard_formulas() {
    for step in -1_000..=2_000 {
      let value = step as f32 / 1_000.0;
      assert_eq!(
        ease_out_cubic(value).to_bits(),
        previous_ease_out(value).to_bits()
      );
      assert_eq!(
        damped_spring(value, 5.0, 6.0).to_bits(),
        previous_pop_spring(value).to_bits()
      );
    }
  }
}
