// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How the library names the files it keeps: a hash of the file a sound was
//! added from and its extension, so the same file added twice is kept once
//! and no name can reach outside the library's folder.

use std::path::Path;

/// The kinds of file the library takes. Whether one holds sound FFmpeg can
/// play is found as it is measured.
pub const EXTENSIONS: [&str; 12] = [
  "aac", "aif", "aiff", "caf", "flac", "m4a", "mp3", "mp4", "oga", "ogg", "opus", "wav",
];
/// How many hex digits a sound's name has before its extension.
const NAME_DIGITS: usize = 32;

/// Whether `name` could be one the library gave out: a content hash and a
/// short extension, nothing that could reach outside a folder.
pub fn is_name(name: &str) -> bool {
  let Some((stem, extension)) = name.split_once('.') else {
    return false;
  };
  is_stem(stem) && EXTENSIONS.contains(&extension)
}

/// Whether `stem` is a content hash, as every file kept for a sound starts.
pub(super) fn is_stem(stem: &str) -> bool {
  stem.len() == NAME_DIGITS
    && stem
      .bytes()
      .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Whether `path` is a kind of file the library takes, by its extension.
pub fn is_sound_file(path: &Path) -> bool {
  extension(path).is_some()
}

pub(super) fn extension(path: &Path) -> Option<String> {
  let extension = path.extension()?.to_str()?.to_ascii_lowercase();
  EXTENSIONS
    .contains(&extension.as_str())
    .then_some(extension)
}

/// A stable 128-bit FNV-1a of `bytes`, as the library's name for them. The
/// name only has to tell sounds apart, not stand up to anyone forging one.
pub(super) fn content_hash(bytes: &[u8]) -> String {
  let mut hash: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
  for byte in bytes {
    hash ^= u128::from(*byte);
    hash = hash.wrapping_mul(0x0000_0000_0100_0000_0000_0000_0000_013b);
  }
  format!("{hash:032x}")
}
