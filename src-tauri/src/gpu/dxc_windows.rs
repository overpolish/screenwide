// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};

const LIBRARY: &str = "dxcompiler.dll";

/// The bundle installs DXC beside the executable, and `pnpm dxc:prepare`
/// puts a copy beside the debug app. Test executables run from `deps`, so a
/// debug build also accepts the prepared copy in `src-tauri/binaries`.
pub(super) fn library_path() -> Result<String, String> {
  let beside = std::env::current_exe()
    .ok()
    .and_then(|executable| executable.parent().map(|directory| directory.join(LIBRARY)));
  let prepared = cfg!(debug_assertions).then(|| {
    Path::new(env!("CARGO_MANIFEST_DIR"))
      .join("binaries")
      .join(LIBRARY)
  });
  [beside, prepared]
    .into_iter()
    .flatten()
    .find(|path| path.is_file())
    .map(|path: PathBuf| path.to_string_lossy().into_owned())
    .ok_or_else(|| {
      if cfg!(debug_assertions) {
        format!("The shader compiler {LIBRARY} is missing; run `pnpm dxc:prepare`")
      } else {
        format!("The shader compiler {LIBRARY} is missing beside Screenwide")
      }
    })
}
