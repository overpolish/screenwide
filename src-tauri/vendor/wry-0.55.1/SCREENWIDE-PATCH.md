# Screenwide patch to Wry 0.55.1

Source: crates.io `wry` 0.55.1. Upstream licenses are preserved.

In `src/wkwebview/mod.rs`, gate macOS application activation during webview
creation on `attributes.focused`. Upstream activates unconditionally, even
for hidden windows built with `.focused(false)`. This raises Settings/Export
before Ruler or OCR has a chance to become main/key.

The overlay presentation code performs activation once the overlay is ready.
Default focused webviews, Windows, and iOS retain their existing behavior.

When upgrading Wry, check whether upstream honors this flag before removing
the Cargo patch. Reproduce with Settings behind two other apps, then launch
and dismiss Ruler and OCR, checking window order and cursor behavior.

Local dependency warning cleanup uses `NSEventModifierFlags` instead of deprecated
modifier aliases and removes redundant unsafe blocks. The deprecated legacy
file-list pasteboard API is allowed only at its import and collection function
to preserve upstream drag/drop compatibility without broad warning suppression.
