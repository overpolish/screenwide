// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The input view laid over the webview's workarea: it takes the pointer,
//! carries the selection chrome's layer, takes the keyboard while a box is
//! typed into, and takes dropped pictures. Every event is handed to the
//! shared engine. The twin of the Windows editor window.

mod drop;
mod keys;
mod pointer;
mod support;
mod typing;

use std::cell::{Cell, RefCell};
use std::sync::Weak;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObjectProtocol, ProtocolObject, Sel};
use objc2::{define_class, msg_send, DefinedClass, MainThreadOnly};
use objc2_app_kit::{
  NSDragOperation, NSDraggingInfo, NSEvent, NSResponder, NSTextInputClient, NSTrackingArea, NSView,
};
use objc2_foundation::{
  NSArray, NSAttributedString, NSAttributedStringKey, NSNotFound, NSObject, NSPoint, NSRange,
  NSRangePointer, NSRect, NSTimer, NSUInteger,
};

use screenwide_workspace::editing::{SurfaceInner, TypingInput};

use support::guard;

pub(super) struct EditorIvars {
  /// The engine behind the view, once the surface is whole.
  engine: RefCell<Weak<SurfaceInner>>,
  tracking: RefCell<Option<Retained<NSTrackingArea>>>,
  /// Whether a box's typing holds the keyboard.
  typing: Cell<bool>,
  /// The caret, in the view's points, where an input method opens.
  caret: Cell<NSPoint>,
  blink: RefCell<Option<Retained<NSTimer>>>,
  /// Whether the drag over the view holds a picture the workspace takes.
  drop_accepts: Cell<bool>,
}

define_class!(
  // Safety: NSView has no subclassing requirements this class breaks, and
  // it implements no `Drop`.
  #[unsafe(super(NSView, NSResponder, NSObject))]
  #[thread_kind = MainThreadOnly]
  #[name = "ScreenwideEditorView"]
  #[ivars = EditorIvars]
  pub(super) struct EditorView;

  impl EditorView {
    #[unsafe(method(isFlipped))]
    fn is_flipped(&self) -> bool {
      true
    }

    #[unsafe(method(acceptsFirstMouse:))]
    fn accepts_first_mouse(&self, _event: Option<&NSEvent>) -> bool {
      true
    }

    #[unsafe(method(acceptsFirstResponder))]
    fn accepts_first_responder(&self) -> bool {
      self.ivars().typing.get()
    }

    #[unsafe(method(resignFirstResponder))]
    fn resign_first_responder(&self) -> bool {
      if self.ivars().typing.get() {
        self.typing(TypingInput::FocusLost);
      }
      true
    }

    #[unsafe(method(updateTrackingAreas))]
    fn update_tracking_areas(&self) {
      self.refresh_tracking();
      // Safety: NSView implements the method.
      let _: () = unsafe { msg_send![super(self), updateTrackingAreas] };
    }

    #[unsafe(method(cursorUpdate:))]
    fn cursor_update(&self, _event: &NSEvent) {
      if let Some(engine) = self.engine() {
        guard(|| engine.refresh_cursor());
      }
    }

    #[unsafe(method(mouseDown:))]
    fn mouse_down(&self, event: &NSEvent) {
      pointer::down(self, event);
    }

    #[unsafe(method(mouseDragged:))]
    fn mouse_dragged(&self, event: &NSEvent) {
      pointer::moved(self, event, true);
    }

    #[unsafe(method(mouseMoved:))]
    fn mouse_moved(&self, event: &NSEvent) {
      pointer::moved(self, event, false);
    }

    #[unsafe(method(mouseUp:))]
    fn mouse_up(&self, event: &NSEvent) {
      pointer::up(self, event);
    }

    #[unsafe(method(rightMouseDown:))]
    fn right_mouse_down(&self, event: &NSEvent) {
      pointer::context_menu(self, event);
    }

    #[unsafe(method(otherMouseDown:))]
    fn other_mouse_down(&self, event: &NSEvent) {
      pointer::pan(self, event, true);
    }

    #[unsafe(method(otherMouseDragged:))]
    fn other_mouse_dragged(&self, event: &NSEvent) {
      pointer::moved(self, event, true);
    }

    #[unsafe(method(otherMouseUp:))]
    fn other_mouse_up(&self, event: &NSEvent) {
      pointer::pan(self, event, false);
    }

    #[unsafe(method(scrollWheel:))]
    fn scroll_wheel(&self, event: &NSEvent) {
      pointer::scroll(self, event);
    }

    #[unsafe(method(magnifyWithEvent:))]
    fn magnify_with_event(&self, event: &NSEvent) {
      pointer::magnify(self, event);
    }

    #[unsafe(method(keyDown:))]
    fn key_down(&self, event: &NSEvent) {
      if !keys::key_down(self, event) {
        // Safety: NSResponder implements the method.
        let _: () = unsafe { msg_send![super(self), keyDown: event] };
      }
    }

    #[unsafe(method(performKeyEquivalent:))]
    fn perform_key_equivalent(&self, event: &NSEvent) -> bool {
      keys::key_equivalent(self, event)
        // Safety: NSView implements the method.
        || unsafe { msg_send![super(self), performKeyEquivalent: event] }
    }

    #[unsafe(method(draggingEntered:))]
    fn dragging_entered(&self, info: &ProtocolObject<dyn NSDraggingInfo>) -> NSDragOperation {
      drop::entered(self, info)
    }

    #[unsafe(method(draggingUpdated:))]
    fn dragging_updated(&self, _info: &ProtocolObject<dyn NSDraggingInfo>) -> NSDragOperation {
      drop::operation(self)
    }

    #[unsafe(method(draggingExited:))]
    fn dragging_exited(&self, _info: Option<&ProtocolObject<dyn NSDraggingInfo>>) {
      self.ivars().drop_accepts.set(false);
    }

    #[unsafe(method(performDragOperation:))]
    fn perform_drag_operation(&self, info: &ProtocolObject<dyn NSDraggingInfo>) -> bool {
      drop::perform(self, info)
    }
  }

  unsafe impl NSObjectProtocol for EditorView {}

  // Text input with no marked text of its own: an input method's
  // composition shows in its own window, opened at the caret, and lands in
  // the box once committed.
  unsafe impl NSTextInputClient for EditorView {
    #[unsafe(method(insertText:replacementRange:))]
    fn insert_text(&self, string: &AnyObject, _replacement: NSRange) {
      keys::insert(self, string);
    }

    #[unsafe(method(doCommandBySelector:))]
    fn do_command_by_selector(&self, selector: Sel) {
      keys::command(self, selector);
    }

    #[unsafe(method(setMarkedText:selectedRange:replacementRange:))]
    fn set_marked_text(&self, _string: &AnyObject, _selected: NSRange, _replacement: NSRange) {}

    #[unsafe(method(unmarkText))]
    fn unmark_text(&self) {}

    #[unsafe(method(selectedRange))]
    fn selected_range(&self) -> NSRange {
      NSRange::new(NSNotFound as usize, 0)
    }

    #[unsafe(method(markedRange))]
    fn marked_range(&self) -> NSRange {
      NSRange::new(NSNotFound as usize, 0)
    }

    #[unsafe(method(hasMarkedText))]
    fn has_marked_text(&self) -> bool {
      false
    }

    #[unsafe(method_id(attributedSubstringForProposedRange:actualRange:))]
    fn attributed_substring(
      &self,
      _range: NSRange,
      _actual: NSRangePointer,
    ) -> Option<Retained<NSAttributedString>> {
      None
    }

    #[unsafe(method_id(validAttributesForMarkedText))]
    fn valid_attributes_for_marked_text(&self) -> Retained<NSArray<NSAttributedStringKey>> {
      NSArray::new()
    }

    #[unsafe(method(firstRectForCharacterRange:actualRange:))]
    fn first_rect_for_character_range(&self, _range: NSRange, _actual: NSRangePointer) -> NSRect {
      self.caret_on_screen()
    }

    #[unsafe(method(characterIndexForPoint:))]
    fn character_index_for_point(&self, _point: NSPoint) -> NSUInteger {
      NSNotFound as NSUInteger
    }
  }
);
