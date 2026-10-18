// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::OnceLock;

/// Cell identifiers in the shared supersampled Lucide alpha atlas. The atlas
/// is decoded once by Rust; macOS and Windows only upload the same R8 pixels.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControlIcon {
  #[default]
  None = 0,
  X = 1,
  Copy = 2,
  Pilcrow = 3,
  RotateCcw = 4,
  Trash2 = 5,
}

pub const ICON_ATLAS_COLUMNS: u32 = 6;
pub const ICON_ATLAS_CELL_SIZE: u32 = 96;

pub struct IconAtlas {
  pixels: Vec<u8>,
  pub width: u32,
  pub height: u32,
}

impl IconAtlas {
  pub fn pixels(&self) -> &[u8] {
    &self.pixels
  }
}

pub fn icon_atlas() -> &'static IconAtlas {
  static ATLAS: OnceLock<IconAtlas> = OnceLock::new();
  ATLAS.get_or_init(|| {
    let image = image::load_from_memory(include_bytes!("icon-atlas.png"))
      .expect("embedded OSC icon atlas must be a valid PNG")
      .into_rgba8();
    let (width, height) = image.dimensions();
    assert_eq!(width, ICON_ATLAS_COLUMNS * ICON_ATLAS_CELL_SIZE);
    assert_eq!(height, ICON_ATLAS_CELL_SIZE);
    IconAtlas {
      pixels: image.pixels().map(|pixel| pixel[3]).collect(),
      width,
      height,
    }
  })
}
