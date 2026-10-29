// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createLucideIcon } from "lucide-react";

// The Frame tool's glyph: a picture in a frame, after SF Symbols'
// `photo.artframe`. The picture is Lucide's own `image` (`rect` from 3 to 21,
// its sun at 9,9 and its mountain) at two thirds, as the Counter glyph takes
// Lucide's numeral at three quarters, set in a frame two pixels clear of it.
// At that size the sun sits closer to the picture's edge than Lucide's two
// pixels; kept, since it is what makes the picture read as one. The frame's
// corners run parallel to the picture's, its radius the picture's and the
// four pixels between them, so the gap is even all the way round.

export const FrameToolIcon = createLucideIcon({
  name: "frame",
  node: [
    [
      "rect",
      { height: "20", key: "frame", rx: "6", width: "20", x: "2", y: "2" },
    ],
    [
      "rect",
      { height: "12", key: "picture", rx: "2", width: "12", x: "6", y: "6" },
    ],
    ["circle", { cx: "10", cy: "10", key: "sun", r: "1.333" }],
    [
      "path",
      {
        d: "m18 14-2.057-2.057a1.333 1.333 0 0 0-1.886 0L8 18",
        key: "mountain",
      },
    ],
  ],
});
