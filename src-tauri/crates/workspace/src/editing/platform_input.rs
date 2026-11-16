// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pointer and typing input a platform hands the editor workspace, and
//! the cursor it is asked to show.

/// The pointer the workspace asks the platform to show.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorKind {
  Arrow,
  /// The arrow tool over empty picture: it draws rather than picks up.
  Crosshair,
  Move,
  ResizeHorizontal,
  ResizeVertical,
  ResizeNesw,
  ResizeNwse,
  /// Over a box being typed into.
  IBeam,
}

/// One pointer event over the workspace, in the drawable's pixels from the
/// viewport's top-left. `centered` is the modifier that resizes about the
/// centre, `snapping` the one that snaps.
#[derive(Clone, Copy, Debug)]
pub enum PointerInput {
  ContextMenu {
    x: f64,
    y: f64,
  },
  DoubleClick {
    x: f64,
    y: f64,
  },
  Down {
    centered: bool,
    x: f64,
    y: f64,
    snapping: bool,
  },
  Move {
    centered: bool,
    x: f64,
    y: f64,
    pressed: bool,
    snapping: bool,
  },
  Cancel,
  /// A button that pans from wherever it lands rather than selecting.
  PanDown {
    x: f64,
    y: f64,
  },
  PanUp {
    x: f64,
    y: f64,
  },
  Up {
    x: f64,
    y: f64,
  },
  /// A wheel turned by `delta` notches over `x`, `y`: zooms about it.
  Wheel {
    x: f64,
    y: f64,
    delta: f64,
  },
  /// A pinch over `x`, `y`, zooming by `factor`.
  Magnify {
    x: f64,
    y: f64,
    factor: f64,
  },
  /// A trackpad scroll, panning the view by logical points.
  Scroll {
    dx: f64,
    dy: f64,
  },
}

/// A key a box being typed into answers, whatever the platform calls it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypingKey {
  Escape,
  Backspace,
  Delete,
  Left,
  Right,
  Home,
  End,
  Up,
  Down,
  /// A letter, by its upper-case character: what a shortcut with Ctrl or
  /// Command held reaches.
  Letter(char),
  Other,
}

/// What the platform hands on to the box being typed into.
#[derive(Clone, Copy, Debug)]
pub enum TypingInput {
  /// A key went down, with whether the shortcut modifier (Ctrl, Command) and
  /// Shift were held.
  Key {
    key: TypingKey,
    shortcut: bool,
    shift: bool,
  },
  /// One UTF-16 unit of typed text, after the keyboard layout and any input
  /// method have had their say.
  Char(u16),
  /// The caret's blink turned over.
  Blink,
  /// The keyboard went elsewhere: to the webview's panel, another window.
  FocusLost,
  /// The box went away from under the typing.
  Finish,
  /// The typing ended while it held the keyboard, which goes back to the
  /// webview.
  Released,
}

/// The modifiers held as the platform last saw them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
  pub shift: bool,
  /// Ctrl on Windows, Command on macOS: what turns a key into a shortcut and
  /// a drag into a snapping one.
  pub shortcut: bool,
}
