// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a key does to the box being typed into: the keys every Windows text
//! field answers, the typed text itself, and the clipboard.

use windows::Win32::{
  Foundation::{GlobalFree, HANDLE, HGLOBAL},
  System::{
    DataExchange::{
      CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
    },
    Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
    Ole::CF_UNICODETEXT,
  },
  UI::Input::KeyboardAndMouse::{
    GetKeyState, VIRTUAL_KEY, VK_BACK, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_HOME, VK_LEFT,
    VK_RIGHT, VK_SHIFT, VK_UP,
  },
};

use super::*;
use crate::editor::annotations::text::typing::Motion;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
  Unchanged,
  /// The caret or the selection moved; the text is as it was.
  Moved,
  Changed,
  Finish,
}

pub(super) fn shift_held() -> bool {
  unsafe { GetKeyState(i32::from(VK_SHIFT.0)) < 0 }
}

/// Applies one key or one typed unit, redraws what it moved, and reports what
/// it changed.
pub(super) fn edit(inner: &Arc<SurfaceInner>, input: editor::TypingInput) {
  let (outcome, change, caret) = {
    let Ok(mut state) = inner.state.lock() else {
      return;
    };
    let place = state
      .annotation
      .typing
      .as_ref()
      .and_then(|typing| place(&state, typing.flat));
    let scale = state.scale;
    let Some(typing) = state.annotation.typing.as_mut() else {
      return;
    };
    let outcome = apply(typing, input, place.as_ref(), inner.editor.typing_window());
    if matches!(outcome, Outcome::Changed | Outcome::Moved) {
      typing.caret_on = true;
    }
    if outcome == Outcome::Changed {
      typing.revision += 1;
    }
    let caret = place.as_ref().map(|place| {
      let (x, y) = place.layout(|layout| typing.edit.caret_point(layout));
      ((place.left + x) * scale, (place.top + y) * scale)
    });
    let change = (outcome == Outcome::Changed).then(|| {
      (
        (typing.layer, typing.index),
        typing.edit.text().to_owned(),
        typing.revision,
      )
    });
    match outcome {
      // Rust re-presents the box with its new text; the marks only have to
      // be in place for that.
      Outcome::Changed => sync_typing_marks(&mut state),
      Outcome::Moved => redraw_composed_panes(inner, &mut state),
      Outcome::Unchanged | Outcome::Finish => {}
    }
    (outcome, change, caret)
  };
  if outcome == Outcome::Finish {
    finish(inner);
    return;
  }
  if matches!(outcome, Outcome::Changed | Outcome::Moved) {
    inner.editor.restart_blink();
    if let Some((x, y)) = caret {
      inner
        .editor
        .place_typing(x.round() as i32, y.round() as i32);
    }
  }
  if let Some((identity, text, revision)) = change {
    report(inner, AnnotationTextPhase::Change, identity, text, revision);
  }
}

fn changed(changed: bool) -> Outcome {
  if changed {
    Outcome::Changed
  } else {
    Outcome::Unchanged
  }
}

fn apply(
  typing: &mut Typing,
  input: editor::TypingInput,
  place: Option<&Place>,
  window: HWND,
) -> Outcome {
  match input {
    editor::TypingInput::Char(unit) => typed(typing, unit),
    editor::TypingInput::Key { key, ctrl, shift } => {
      pressed(typing, VIRTUAL_KEY(key), ctrl, shift, place, window)
    }
    _ => Outcome::Unchanged,
  }
}

/// One typed unit. Return arrives as a carriage return and is a line break;
/// every other control character is a key already handled, or one a box
/// cannot show.
fn typed(typing: &mut Typing, unit: u16) -> Outcome {
  if (0xD800..0xDC00).contains(&unit) {
    typing.surrogate = Some(unit);
    return Outcome::Unchanged;
  }
  let units = match typing.surrogate.take() {
    Some(high) if (0xDC00..0xE000).contains(&unit) => vec![high, unit],
    _ => vec![unit],
  };
  let text = String::from_utf16_lossy(&units);
  match text.as_str() {
    "\r" | "\n" => changed(typing.edit.insert("\n")),
    text if text.chars().all(char::is_control) => Outcome::Unchanged,
    text => changed(typing.edit.insert(text)),
  }
}

fn pressed(
  typing: &mut Typing,
  key: VIRTUAL_KEY,
  ctrl: bool,
  shift: bool,
  place: Option<&Place>,
  window: HWND,
) -> Outcome {
  let mut go = |motion: Motion| {
    typing.edit.go(motion, shift);
    Outcome::Moved
  };
  match key {
    // Escape keeps the typing: undo is there for a change that was not
    // wanted.
    VK_ESCAPE => Outcome::Finish,
    VK_BACK => changed(typing.edit.erase(true, ctrl)),
    VK_DELETE => changed(typing.edit.erase(false, ctrl)),
    VK_LEFT => go(if ctrl { Motion::WordLeft } else { Motion::Left }),
    VK_RIGHT => go(if ctrl {
      Motion::WordRight
    } else {
      Motion::Right
    }),
    VK_HOME => go(if ctrl {
      Motion::Start
    } else {
      Motion::LineStart
    }),
    VK_END => go(if ctrl { Motion::End } else { Motion::LineEnd }),
    VK_UP | VK_DOWN => place.map_or(Outcome::Unchanged, |place| {
      place.layout(|layout| typing.edit.go_line(key == VK_DOWN, shift, layout));
      Outcome::Moved
    }),
    _ if ctrl => shortcut(typing, key.0, shift, window),
    _ => Outcome::Unchanged,
  }
}

fn shortcut(typing: &mut Typing, key: u16, shift: bool, window: HWND) -> Outcome {
  match u8::try_from(key).map(char::from) {
    Ok('A') => {
      typing.edit.select_all();
      Outcome::Moved
    }
    Ok('C') => {
      copy(window, typing.edit.selected());
      Outcome::Unchanged
    }
    Ok('X') => {
      copy(window, typing.edit.selected());
      changed(typing.edit.erase(true, false))
    }
    Ok('V') => paste(window).map_or(Outcome::Unchanged, |text| changed(typing.edit.paste(&text))),
    Ok('Z') if shift => changed(typing.edit.redo()),
    Ok('Z') => changed(typing.edit.undo()),
    Ok('Y') => changed(typing.edit.redo()),
    _ => Outcome::Unchanged,
  }
}

/// Puts `text` on the clipboard, with the line breaks Windows text uses.
fn copy(window: HWND, text: &str) {
  if text.is_empty() {
    return;
  }
  let wide: Vec<u16> = text
    .replace('\n', "\r\n")
    .encode_utf16()
    .chain([0])
    .collect();
  unsafe {
    if OpenClipboard(Some(window)).is_err() {
      return;
    }
    let _ = EmptyClipboard();
    if let Ok(memory) = GlobalAlloc(GMEM_MOVEABLE, wide.len() * 2) {
      let target = GlobalLock(memory).cast::<u16>();
      if target.is_null() {
        let _ = GlobalFree(Some(memory));
      } else {
        std::ptr::copy_nonoverlapping(wide.as_ptr(), target, wide.len());
        let _ = GlobalUnlock(memory);
        // The clipboard owns the memory once it takes it, and only then.
        if SetClipboardData(u32::from(CF_UNICODETEXT.0), Some(HANDLE(memory.0))).is_err() {
          let _ = GlobalFree(Some(memory));
        }
      }
    }
    let _ = CloseClipboard();
  }
}

/// The text on the clipboard, if there is any.
fn paste(window: HWND) -> Option<String> {
  unsafe {
    OpenClipboard(Some(window)).ok()?;
    let text = GetClipboardData(u32::from(CF_UNICODETEXT.0))
      .ok()
      .and_then(|handle| {
        let memory = HGLOBAL(handle.0);
        let source = GlobalLock(memory).cast::<u16>();
        if source.is_null() {
          return None;
        }
        let mut length = 0;
        while *source.add(length) != 0 {
          length += 1;
        }
        let text = String::from_utf16_lossy(std::slice::from_raw_parts(source, length));
        let _ = GlobalUnlock(memory);
        Some(text)
      });
    let _ = CloseClipboard();
    text
  }
}
