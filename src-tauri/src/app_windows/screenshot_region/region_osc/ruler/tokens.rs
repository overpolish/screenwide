// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The design tokens the Ruler's labels and callout are drawn to, in points,
//! from `src/styles/theme.css`. One look on every platform, as the island it
//! matches has, so none of them follow the Windows control skin.

/// `--spacing-control-height`: a label's height.
pub(super) const CONTROL_HEIGHT: f64 = 28.0;
/// `--spacing-inset`: from a label's edge to its text, and from the
/// callout's edges to its swatch and text.
pub(super) const INSET: f64 = 12.0;
/// `--spacing-inset-sm`: above and below the callout's two lines.
pub(super) const INSET_SM: f64 = 6.0;
/// `--spacing-inline`: between a label and what it names, and between the
/// callout's swatch and the text it goes with.
pub(super) const INLINE: f64 = 6.0;
/// `--spacing-separate`: the least room between a label and the screen's
/// edge, and between the pointer and the callout.
pub(super) const SEPARATE: f64 = 12.0;
/// `--spacing-icon`: the callout's swatch, the size of the annotate island's
/// colour.
pub(super) const ICON: f64 = 16.0;
/// `--text-caption`, the role for readouts: every figure the Ruler shows.
pub(super) const CAPTION_SIZE: f64 = 11.0;
pub(super) const CAPTION_LINE: f64 = 14.0;
/// The callout: its two caption lines with the small inset above and below.
pub(super) const CALLOUT_HEIGHT: f64 = CAPTION_LINE * 2.0 + INSET_SM * 2.0;
