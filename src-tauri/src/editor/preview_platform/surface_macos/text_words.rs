// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The typing of a box whose look sets chosen words apart, handed to the
//! text view that takes its keys. The view sets every word in one face, so
//! it asks here where a press lands and where a line up or down goes, as
//! the compositor lays the words out. Everything here runs on the main
//! thread, which owns the view; the view frees what it was handed when the
//! typing ends.

use std::ffi::c_void;

use screenwide_annotations::text::typed::TypedWords;

/// The typing `words` points at, or none.
///
/// # Safety
/// `words` is null or came from [`into_handle`] and is not yet freed.
unsafe fn typed<'a>(words: *mut c_void) -> Option<&'a mut TypedWords> {
  unsafe { words.cast::<TypedWords>().as_mut() }
}

/// `typed` handed over to the text view, or null where there is none.
pub(super) fn into_handle(typed: Option<TypedWords>) -> *mut c_void {
  typed.map_or(std::ptr::null_mut(), |typed| {
    Box::into_raw(Box::new(typed)).cast()
  })
}

/// Carries the chosen words through the change that left the `length`
/// bytes of UTF-8 at `text`.
///
/// # Safety
/// `words` is as [`typed`] takes it, and `text` points at `length` bytes.
#[no_mangle]
pub unsafe extern "C" fn screenwide_text_words_follow(
  words: *mut c_void,
  text: *const u8,
  length: u32,
) {
  let Some(typed) = (unsafe { typed(words) }) else {
    return;
  };
  let bytes = if text.is_null() {
    &[][..]
  } else {
    unsafe { std::slice::from_raw_parts(text, length as usize) }
  };
  typed.follow(&String::from_utf8_lossy(bytes));
}

/// Non-zero while a chosen word is left to set apart.
///
/// # Safety
/// `words` is as [`typed`] takes it.
#[no_mangle]
pub unsafe extern "C" fn screenwide_text_words_sets_apart(words: *mut c_void) -> i32 {
  i32::from(unsafe { typed(words) }.is_some_and(|typed| typed.sets_apart()))
}

/// The byte of the text nearest `(x, y)`, from the top-left of its block,
/// with its type set at `font` and its lines lined up by `align`.
///
/// # Safety
/// `words` is as [`typed`] takes it.
#[no_mangle]
pub unsafe extern "C" fn screenwide_text_words_offset_at(
  words: *mut c_void,
  font: f64,
  align: u32,
  x: f64,
  y: f64,
) -> u32 {
  unsafe { typed(words) }.map_or(0, |typed| {
    u32::try_from(typed.offset_at(font, align, (x, y))).unwrap_or(u32::MAX)
  })
}

/// The byte a line up or down from byte `offset` goes to, keeping to
/// `*goal` across the block where it is a number, which then holds the goal
/// the next move keeps to.
///
/// # Safety
/// `words` is as [`typed`] takes it, and `goal` points at a writable number.
#[no_mangle]
pub unsafe extern "C" fn screenwide_text_words_line(
  words: *mut c_void,
  font: f64,
  align: u32,
  offset: u32,
  down: i32,
  goal: *mut f64,
) -> u32 {
  let Some(typed) = (unsafe { typed(words) }) else {
    return offset;
  };
  let Some(goal) = (unsafe { goal.as_mut() }) else {
    return offset;
  };
  let kept = goal.is_finite().then_some(*goal);
  let (moved, next) = typed.line_from((font, align), offset as usize, down != 0, kept);
  *goal = next;
  u32::try_from(moved).unwrap_or(u32::MAX)
}

/// Frees the typing the text view was handed.
///
/// # Safety
/// `words` is null or came from [`into_handle`], and is freed once.
#[no_mangle]
pub unsafe extern "C" fn screenwide_text_words_free(words: *mut c_void) {
  if !words.is_null() {
    drop(unsafe { Box::from_raw(words.cast::<TypedWords>()) });
  }
}
