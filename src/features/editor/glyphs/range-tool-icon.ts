// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createLucideIcon } from "lucide-react";

// The Range tool's glyph: a clip held between an in and an out bracket, as
// video editors draw their range tools. The brackets keep Lucide's two
// pixels from the box, with corners of one as `brackets` has; the clip
// stands four pixels clear of their uprights, its ends level with the
// brackets' arms so the three read as one stretch.

export const RangeToolIcon = createLucideIcon({
  name: "range",
  node: [
    ["path", { d: "M8 4H5a1 1 0 0 0-1 1v14a1 1 0 0 0 1 1h3", key: "in" }],
    ["path", { d: "M16 4h3a1 1 0 0 1 1 1v14a1 1 0 0 1-1 1h-3", key: "out" }],
    ["rect", { height: "6", key: "clip", rx: "1", width: "8", x: "8", y: "9" }],
  ],
});
