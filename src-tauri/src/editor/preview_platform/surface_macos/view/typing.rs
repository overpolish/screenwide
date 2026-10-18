// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The keyboard a box's typing takes: the view becomes the window's first
//! responder for as long as the typing lasts, with the caret's blink on a
//! timer, and hands it back when it ends. The twin of the Windows typing
//! window.

use std::ptr::NonNull;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::Message;
use objc2_app_kit::NSResponder;
use objc2_foundation::{NSPoint, NSTimer};

use objc2::DefinedClass;

use super::EditorView;
use screenwide_workspace::editing::TypingInput;

/// How long the caret shows and hides for, as AppKit's own text views blink.
const BLINK_SECONDS: f64 = 0.5;

impl EditorView {
  /// Takes the keyboard for a box being typed into.
  pub(in crate::editor::preview_platform::surface_macos) fn begin_typing(&self) {
    self.ivars().typing.set(true);
    if let Some(window) = self.window() {
      let responder: &NSResponder = self;
      window.makeFirstResponder(Some(responder));
    }
    self.restart_blink();
  }

  /// Gives the keyboard back once the typing is over.
  pub(in crate::editor::preview_platform::surface_macos) fn end_typing(&self) {
    self.stop_blink();
    if !self.ivars().typing.replace(false) {
      return;
    }
    let focused = self.window().is_some_and(|window| {
      window
        .firstResponder()
        .is_some_and(|responder| std::ptr::eq(&*responder, self as &NSResponder))
    });
    if focused {
      self.typing(TypingInput::Released);
    }
  }

  /// Ends the typing and reports what was typed: the box went away while
  /// something up the stack may still have held the surface.
  pub(in crate::editor::preview_platform::surface_macos) fn finish_typing(&self) {
    self.typing(TypingInput::Finish);
  }

  /// Shows the caret for a whole blink again, as after every edit.
  pub(in crate::editor::preview_platform::surface_macos) fn restart_blink(&self) {
    self.stop_blink();
    if !self.ivars().typing.get() {
      return;
    }
    let view: Retained<EditorView> = self.retain();
    let tick = RcBlock::new(move |_timer: NonNull<NSTimer>| {
      view.typing(TypingInput::Blink);
    });
    // Safety: the block lives as long as the timer, which is invalidated
    // when the typing ends or the blink restarts.
    let timer =
      unsafe { NSTimer::scheduledTimerWithTimeInterval_repeats_block(BLINK_SECONDS, true, &tick) };
    *self.ivars().blink.borrow_mut() = Some(timer);
  }

  /// Puts the caret, in the view's points, where an input method opens.
  pub(in crate::editor::preview_platform::surface_macos) fn place_caret(&self, point: NSPoint) {
    self.ivars().caret.set(point);
  }

  fn stop_blink(&self) {
    if let Some(timer) = self.ivars().blink.borrow_mut().take() {
      timer.invalidate();
    }
  }
}
