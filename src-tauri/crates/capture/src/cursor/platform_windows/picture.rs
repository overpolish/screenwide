// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A custom cursor's picture, drawn by Windows to be kept beside the
//! recording so it is drawn as it was.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hasher};
use std::sync::Arc;

use windows::Win32::UI::WindowsAndMessaging::HCURSOR;

use super::super::CursorPicture;

/// How many cursors' pictures are remembered while recording; a real
/// working set is a handful, so a full flush past it costs little.
const REMEMBERED: usize = 64;

/// The pictures already drawn, by cursor, so moving between an app's
/// cursors draws each once.
#[derive(Default)]
pub(super) struct Pictures {
  drawn: HashMap<isize, Option<CursorPicture>>,
}

impl Pictures {
  /// `cursor`'s picture, `side` pixels square.
  pub(super) fn of(&mut self, cursor: HCURSOR, side: u32) -> Option<CursorPicture> {
    let key = cursor.0 as isize;
    if let Some(known) = self.drawn.get(&key) {
      return known.clone();
    }
    if self.drawn.len() >= REMEMBERED {
      self.drawn.clear();
    }
    let picture = kept(cursor, side);
    self.drawn.insert(key, picture.clone());
    picture
  }
}

fn kept(cursor: HCURSOR, side: u32) -> Option<CursorPicture> {
  let picture = screenwide_cursor_effects::drawn_windows_cursor(cursor, side).ok()?;
  let mut hasher = DefaultHasher::new();
  hasher.write_u32(side);
  hasher.write(picture.as_raw());
  let mut png = Vec::new();
  picture
    .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
    .ok()?;
  Some(CursorPicture {
    name: format!("{:016x}", hasher.finish()),
    png: Arc::from(png),
  })
}
