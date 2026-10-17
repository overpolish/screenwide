// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The text engine: how type is set and measured on each platform, the style
//! vocabulary text is dressed in, and the templates that name a style. A
//! template is written once in that vocabulary and dresses any text that
//! takes a style; [`captions`] is the text a recording's transcript makes.

mod baseline;
pub mod captions;
mod controls;
mod elements;
pub mod emphasis;
pub mod engine;
mod font;
mod layout;
mod motion;
mod shadow;
mod size;
mod style;
mod templates;

pub use controls::{BoxPoint, ControlLimits, ControlSpace, TextControl, MAX_REACH};
pub use elements::{TailShape, TextElement};
pub use emphasis::TextSpan;
pub use font::{TextFont, MAX_WEIGHT, MIN_WEIGHT};
pub use layout::TextLayout;
pub use motion::{
  TextEasing, TextKeyframe, TextMotion, TextMotionTrack, TextProperty, MAX_KEYFRAMES, MAX_MOTION_MS,
};
pub use shadow::TextShadow;
pub use size::TextSize;
pub use style::{
  rgba, TextAccent, TextArrangement, TextBackground, TextFace, TextLines, TextStyle, AUTO_INK,
  MAX_ACCENT_SIZE_PERCENT, MAX_SIZE_PERCENT, MAX_WORDS, MIN_ACCENT_SIZE_PERCENT, MIN_SIZE_PERCENT,
  MIN_WORDS,
};
pub use templates::{
  bundled_templates, load_user_templates, parse_manifest, remove_user_template,
  template_font_files, text_templates, user_template_fonts, ManifestError, TemplateFont,
  TemplatePackage, TextPreviewBackdrop, TextTemplate, TextTemplateId, TextTemplateInfo,
  TextTemplateSource, FORMAT_VERSION, MANIFEST_FILE,
};
