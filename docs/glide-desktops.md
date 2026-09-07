<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Glide Spaces checkpoint

On macOS, start over a window titlebar with the configurable Spaces modifier held (Option by default):

- Mouse: also hold the mouse Glide modifier (Command by default), then move left/right. Release either modifier to commit.
- Trackpad: swipe left/right with two fingers. Lift to commit.
- Wheel: scroll up/down for previous/next. The scroll gesture ends after 300ms without wheel input, or when the modifier is released.

The preview uses a separate native window for every ordinary Space on that display group. A hidden pool is preloaded at app startup, sized for the largest display group, and reused by gestures. Each is the same size, material and shadow as normal Glide, with no title highlight. The primary surface and app icon show the selected destination. Selection does not move the real window. After each step, 100ms of stillness rearms selection, with small jitter tolerated and readiness feedback synchronized with the haptic setting. Further steps change the pending destination. Return to the original Space or press Escape to cancel. Gesture end dismisses the previews and carries the actual window and user to the selected Space together. Completion and late icon updates do not reopen the previews.

The shared desktop contract uses opaque IDs, live ordering and membership, one native carry operation, and transition state. It skips fullscreen Spaces and never wraps or crosses display groups. The exact previewed destination is revalidated on release; stale targets, sticky windows, active transitions, and removed destinations are rejected. Windows can implement this same adapter independently.

The macOS adapter reads live Space membership and animation state through dynamically resolved SkyLight APIs. It uses the titlebar position captured by the real Glide gesture, a native titlebar hold, and HID Control–Arrow events. The Mission Control Control–Left/Right shortcuts must be enabled. Interactive titlebar controls and an already-held mouse button are rejected. Glide owns cursor hiding/pinning throughout the carry and releases the synthetic hold before restoring the pointer. Completion requires both window membership and current desktop to agree after animation settles.

## Native checks

With at least two ordinary Spaces, select a neighbour and keep the gesture held: only the preview should change. Release and confirm the window stays anchored as desktops slide beneath it. Check both directions in Safari and Terminal, and confirm the pointer stays at its captured position. Repeat with mouse, trackpad, and wheel.

Try selecting multiple Spaces before release, reversing back to the original Space, Escape, and reaching either end. Fullscreen Spaces should be skipped. Repeat on each monitor with separate Spaces enabled; when disabled, the desktop group spans the displays. Every preview should have its own native background and shadow, with the desktop visible between windows. Normal resize previews should look identical to before.

Automated policy tests and browser stories do not prove live window movement or native transparency; those require this user-testable checkpoint. Existing resize timing remains unchanged.
