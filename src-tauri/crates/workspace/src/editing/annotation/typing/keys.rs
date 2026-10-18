// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a key does to the box being typed into: the keys every text field
//! answers, the typed text itself, and the clipboard.

use super::*;
use screenwide_text::engine::typing::Motion;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
  Unchanged,
  /// The caret or the selection moved; the text is as it was.
  Moved,
  Changed,
  Finish,
}

/// Applies one key or one typed unit, redraws what it moved, and reports what
/// it changed.
pub(super) fn edit(inner: &Arc<SurfaceInner>, input: TypingInput) {
  let (outcome, change, caret) = {
    let Ok(mut state) = inner.state.lock() else {
      return;
    };
    let place = state
      .annotation
      .typing
      .as_ref()
      .and_then(|typing| place(&state, typing.target, &typing.face));
    let scale = state.scale;
    let Some(typing) = state.annotation.typing.as_mut() else {
      return;
    };
    let outcome = apply(typing, input, place.as_ref(), inner.host.as_ref());
    if matches!(outcome, Outcome::Changed | Outcome::Moved) {
      typing.caret_on = true;
    }
    if outcome == Outcome::Changed {
      typing.revision += 1;
      if let Some(words) = typing.words.as_mut() {
        words.follow(typing.edit.text());
      }
    }
    let caret = place.as_ref().map(|place| {
      let (x, y) = place.layout(typing.words.as_ref(), |lines| {
        typing.edit.caret_point(lines)
      });
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
      Outcome::Changed => sync_typing_marks(inner, &mut state),
      Outcome::Moved => redraw_workspace(inner, &mut state),
      Outcome::Unchanged | Outcome::Finish => {}
    }
    (outcome, change, caret)
  };
  if outcome == Outcome::Finish {
    finish(inner);
    return;
  }
  if matches!(outcome, Outcome::Changed | Outcome::Moved) {
    inner.host.restart_blink();
    if let Some((x, y)) = caret {
      inner.host.place_typing(x.round() as i32, y.round() as i32);
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
  input: TypingInput,
  place: Option<&Place>,
  host: &dyn Host,
) -> Outcome {
  match input {
    TypingInput::Char(unit) => typed(typing, unit),
    TypingInput::Key {
      key,
      shortcut,
      shift,
    } => pressed(typing, key, shortcut, shift, place, host),
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
    // Return ends the typing of a caption, as it does in a field: a
    // caption's lines are its rules' and the breaks placed in the panel.
    "\r" | "\n" if typing.is_caption() => Outcome::Finish,
    "\r" | "\n" => changed(typing.edit.insert("\n")),
    text if text.chars().all(char::is_control) => Outcome::Unchanged,
    text => changed(typing.edit.insert(text)),
  }
}

fn pressed(
  typing: &mut Typing,
  key: TypingKey,
  ctrl: bool,
  shift: bool,
  place: Option<&Place>,
  host: &dyn Host,
) -> Outcome {
  let mut go = |motion: Motion| {
    typing.edit.go(motion, shift);
    Outcome::Moved
  };
  match key {
    // Escape keeps the typing: undo is there for a change that was not
    // wanted.
    TypingKey::Escape => Outcome::Finish,
    TypingKey::Backspace => changed(typing.edit.erase(true, ctrl)),
    TypingKey::Delete => changed(typing.edit.erase(false, ctrl)),
    TypingKey::Left => go(if ctrl { Motion::WordLeft } else { Motion::Left }),
    TypingKey::Right => go(if ctrl {
      Motion::WordRight
    } else {
      Motion::Right
    }),
    TypingKey::Home => go(if ctrl {
      Motion::Start
    } else {
      Motion::LineStart
    }),
    TypingKey::End => go(if ctrl { Motion::End } else { Motion::LineEnd }),
    TypingKey::Up | TypingKey::Down => place.map_or(Outcome::Unchanged, |place| {
      place.layout(typing.words.as_ref(), |lines| {
        typing.edit.go_line(key == TypingKey::Down, shift, lines)
      });
      Outcome::Moved
    }),
    TypingKey::Letter(letter) if ctrl => shortcut(typing, letter, shift, host),
    _ => Outcome::Unchanged,
  }
}

fn shortcut(typing: &mut Typing, letter: char, shift: bool, host: &dyn Host) -> Outcome {
  match letter {
    'A' => {
      typing.edit.select_all();
      Outcome::Moved
    }
    'C' => {
      copy(host, typing.edit.selected());
      Outcome::Unchanged
    }
    'X' => {
      copy(host, typing.edit.selected());
      changed(typing.edit.erase(true, false))
    }
    'V' => host
      .clipboard_text()
      .map_or(Outcome::Unchanged, |text| changed(typing.edit.paste(&text))),
    'Z' if shift => changed(typing.edit.redo()),
    'Z' => changed(typing.edit.undo()),
    'Y' => changed(typing.edit.redo()),
    _ => Outcome::Unchanged,
  }
}

fn copy(host: &dyn Host, text: &str) {
  if !text.is_empty() {
    host.set_clipboard_text(text);
  }
}
