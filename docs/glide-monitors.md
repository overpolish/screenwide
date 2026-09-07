<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Glide monitor movement checkpoint

On macOS with multiple extended displays, hold the configurable Monitors control (Z by default) over a titlebar to select monitor movement. Move the mouse or use a two-finger trackpad gesture toward a display. The first deliberate push selects a destination directly, including diagonals. Normal Glide retains its resize/minimize ladder, and the Spaces control continues to select virtual desktops.

Further steps select adjacent monitors after the usual readiness pause. A missing neighbour holds the selection; there is no wrap. Each monitor gets a separate native preview window of the usual 48×32 size. Positions approximate the actual display arrangement without scaling the previews to monitor size. Neutral fill marks the original monitor when another is selected; primary fill and the app icon show the destination. Haptics and the readiness pulse share the same stillness gate.

Releasing the control commits a mouse move; lifting commits a trackpad move. A directional scroll wheel also supplies monitor steps while the control is held, with release committing. The window keeps its original size and relative placement on the destination work area. Oversized windows keep their size with their top-left corner accessible. Selecting the original monitor or pressing Escape leaves the window unchanged.

Glide and the recording source selector share monitor enumeration, capture IDs and arrangement coordinates. Shared Rust policy owns neighbour selection, gesture intent, placement and equal-sized previews; the macOS adapter supplies native geometry and applies the final frame. Windows keeps its existing behaviour until its adapter is connected.

## Native verification

Test Z with mouse movement and two-finger gestures for side-by-side, above/below and diagonal arrangements. The window should stay unchanged until release, then move without resizing. Check reversal to the origin, Escape, control release before finger lift, momentum tails and disconnected destinations. Test mixed display sizes/scales and cursor-follow enabled and disabled. Confirm ordinary resizing, minimizing and Spaces gestures still work.

On a single display monitor mode does nothing. Automated tests cover policy; native input, placement, cursor landing and materials require live verification.
