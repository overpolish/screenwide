// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which tool is in hand, as a mode the native chrome reads, and what each
//! mode draws.

#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::AnnotationKind;

/// What the pointer does over the picture while a tool is in hand. The select
/// tool hit-tests the annotations already there and lets every other press fall
/// through to the layer; a drawing tool also makes a new annotation on empty
/// picture. The values are the native `ScreenwideAnnotationMode`.
pub const MODE_NONE: u32 = 0;
pub const MODE_SELECT: u32 = 1;
pub(crate) const MODE_ARROW: u32 = 2;
pub(crate) const MODE_COUNTER: u32 = 3;
pub const MODE_TEXT: u32 = 4;
pub(crate) const MODE_REDACT: u32 = 5;
pub const MODE_HIGHLIGHT: u32 = 6;
pub(crate) const MODE_SHAPE: u32 = 7;
pub(crate) const MODE_SPOTLIGHT: u32 = 8;
pub const MODE_DRAW: u32 = 9;
pub(crate) const MODE_MAGNIFY: u32 = 10;
/// The marquee: it picks annotations up as the select tool does, and a drag
/// over empty picture draws a band that chooses everything it touches.
pub const MODE_MARQUEE: u32 = 11;
pub const MODE_IMAGE: u32 = 12;
pub(crate) const MODE_FOCUS: u32 = 13;

/// The tool name React sends, as a mode. Anything else puts the chrome away.
pub fn annotation_mode(tool: Option<&str>) -> u32 {
  match tool {
    Some("arrow") => MODE_ARROW,
    Some("counter") => MODE_COUNTER,
    Some("text") => MODE_TEXT,
    Some("redact") => MODE_REDACT,
    Some("highlight") => MODE_HIGHLIGHT,
    Some("shape") => MODE_SHAPE,
    Some("spotlight") => MODE_SPOTLIGHT,
    Some("draw") => MODE_DRAW,
    Some("magnify") => MODE_MAGNIFY,
    Some("image") => MODE_IMAGE,
    Some("focus") => MODE_FOCUS,
    Some("select") => MODE_SELECT,
    Some("marquee") => MODE_MARQUEE,
    _ => MODE_NONE,
  }
}

/// Whether the tool in hand halos nothing: with no tool there is nothing to
/// pick, and the pen picks nothing up. Taking one up retires any halo, since
/// the pointer may never move again to do it.
pub fn hovers_nothing(mode: u32) -> bool {
  mode == MODE_NONE || mode == MODE_DRAW
}

/// The shape the tool in hand draws, or `None` where the tool draws nothing.
/// This is also the one answer to "does this mode draw at all": both hosts
/// ask before a press on empty picture may become an annotation, so a
/// [`super::AnnotationGestureTarget::New`] under any other mode never
/// arrives.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub fn drawing_kind(mode: u32) -> Option<AnnotationKind> {
  match mode {
    MODE_ARROW => Some(AnnotationKind::Arrow),
    MODE_COUNTER => Some(AnnotationKind::Counter),
    MODE_TEXT => Some(AnnotationKind::Text),
    MODE_REDACT => Some(AnnotationKind::Redact),
    MODE_HIGHLIGHT => Some(AnnotationKind::Highlight),
    MODE_SHAPE => Some(AnnotationKind::Shape),
    MODE_SPOTLIGHT => Some(AnnotationKind::Spotlight),
    MODE_DRAW => Some(AnnotationKind::Draw),
    MODE_MAGNIFY => Some(AnnotationKind::Magnify),
    MODE_IMAGE => Some(AnnotationKind::Image),
    MODE_FOCUS => Some(AnnotationKind::Focus),
    _ => None,
  }
}
