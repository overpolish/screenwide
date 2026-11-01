// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { EditorKind } from "../../types";

// Which island the editor asks its island window for. What it holds comes
// from the tool panel snapshot the editor publishes: an annotation island
// shows the annotation in hand, or the pen's next one, or how many the
// marquee chose; the Crop tool's shows the crop of the layer in hand; each
// of the title bar's modes opens its own. Captions
// opens the Captions island with the Script island at the opposite corner,
// and Transcript the Script island alone, in the same place. A beat, lull,
// moment or range chosen on the strip brings out its own, and several
// beats and lulls gathered with Shift the strip group's, over what is
// chosen at the picture's foot; the transport's Tracks button brings out
// the Tracks island.

export type EditorIslandKind =
  | "annotation"
  | "beat"
  | "camera"
  | "captions"
  | "crop"
  | "cursor"
  | "frame"
  | "keyboard"
  | "lull"
  | "moment"
  | "range"
  | "scene"
  | "screen"
  | "sound"
  | "stripGroup"
  | "tracks"
  | "transcript";

/** The islands of something chosen on the strip, which stand over it. */
const STRIP_ISLANDS: readonly EditorIslandKind[] = [
  "beat",
  "lull",
  "moment",
  "range",
  "stripGroup",
];

export const isStripIsland = (island: EditorIslandKind | null) =>
  island !== null && STRIP_ISLANDS.includes(island);

/** The editor's island, over the picture of the editor that owns it: which
 * island it is, put away when null. It stands at the picture's top end
 * corner, but for a strip item's, which stands over the item. The panel
 * window covers the picture and lets clicks through everywhere but the
 * island. */
export type PopupPanelIslandContent = {
  island: EditorIslandKind | null;
  kind: "island";
  workspace: EditorKind;
  /** How far along the picture the chosen strip items' middle sits, in px
   * from its left edge: where a strip item's island stands. The items
   * carry `data-strip-island` for it to find them. */
  x: number;
};
