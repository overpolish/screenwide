<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Local dependency patches

Keep a short entry here whenever adding or changing a vendored dependency patch. Cargo overrides live in [`../Cargo.toml`](../Cargo.toml) under `[patch.crates-io]`. Line numbers refer to the local vendored files; update them when a patch changes. Preserve each dependency's upstream license files.

## Wry 0.55.1

**Source:** crates.io `wry` 0.55.1, vendored in `wry-0.55.1/`.

**Reason:** Creating a macOS webview unconditionally activated Screenwide, even with `.focused(false)`. Ruler and OCR create hidden webviews, so Settings/Export rose above other apps before the overlay was ready. The patch respects the focus flag; Screenwide's overlay presentation code activates the app after selecting the overlay as main/key. Windows and iOS behavior is unchanged.

All paths below are relative to `wry-0.55.1/src/wkwebview/`.

| File | Local lines | Change |
| --- | --- | --- |
| [mod.rs](wry-0.55.1/src/wkwebview/mod.rs) | 691–704 | Gate macOS activation on `attributes.focused`. |
| [synthetic_mouse_events.rs](wry-0.55.1/src/wkwebview/synthetic_mouse_events.rs) | 1, 113–116 | Replace deprecated modifier aliases with `NSEventModifierFlags`. |
| [drag_drop.rs](wry-0.55.1/src/wkwebview/drag_drop.rs) | 12–14, 21 | Narrow deprecation exemptions preserve upstream legacy file-list pasteboard compatibility. |
| [drag_drop.rs](wry-0.55.1/src/wkwebview/drag_drop.rs) | 44, 61, 89 | Remove redundant `unsafe` blocks around `draggingLocation()`. |
| [class/wry_web_view_parent.rs](wry-0.55.1/src/wkwebview/class/wry_web_view_parent.rs) | 33–35 | Remove redundant `unsafe` around menu key handling. |
| [class/wry_web_view_ui_delegate.rs](wry-0.55.1/src/wkwebview/class/wry_web_view_ui_delegate.rs) | 43 | Remove redundant `unsafe` around `removeFromSuperview()`. |
| [mod.rs](wry-0.55.1/src/wkwebview/mod.rs) | 1353 | Remove redundant `unsafe` around `absoluteString()`. |

**Validation:** macOS `cargo build` completed without warnings. User confirmed Quick Screenshot, Ruler, and OCR preserve other windows' order on launch and dismissal with the accompanying Screenwide focus-management fixes.

**Upgrade/removal:** Check whether upstream now honors `.focused(false)` on macOS. Once it does, remove the Wry Cargo override and vendored copy, update `Cargo.lock`, and retest: place Settings behind two other apps, launch and dismiss/complete each overlay, and check window order and cursor changes. Recheck compiler warnings against the replacement version.

## Template for future patches

### Dependency name and version

- **Source:** Upstream release/commit and local directory.
- **Reason:** Bug or compatibility requirement motivating the patch.
- **Changes:** File paths, local line numbers, and a brief description of each change.
- **Validation:** Checks run and any manual reproduction used.
- **Upgrade/removal:** Upstream issue/PR if available, removal condition, and retest steps.
