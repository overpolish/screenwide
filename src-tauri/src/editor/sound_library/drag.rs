// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A sound dragged out of the library as its file, so it lands on the
//! editor's lanes as a sound dragged in from the Finder does, and in any
//! other app that takes files. It is drawn under the pointer as the page
//! draws it, rather than as a file's icon.

use screenwide_sound_track::store;

use crate::editor::save::{drag_out, DragLook};

/// Starts dragging the library's file of the sound `name` out of the calling
/// window, from the press its page is handling now, drawn as `png`, `width`
/// by `height` points.
#[tauri::command]
pub fn drag_library_sound(
  window: tauri::WebviewWindow,
  name: String,
  png: Vec<u8>,
  width: f64,
  height: f64,
) -> Result<(), String> {
  let (path, _) =
    store::resolve(None, &name).ok_or_else(|| "That sound is not in the library".to_owned())?;
  drag_out(&window, &path, &DragLook::Png { height, png, width })
}
