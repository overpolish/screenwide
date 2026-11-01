// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ISLAND } from "../../../primitives/island/island-geometry";

import { EditorIsland, type EditorIslandContent } from "./editor-island";

/** An editor island as its window shows it, hung at the top of the canvas
 * as it hangs under what opened it. Laid out left to right, like the strip
 * and the picture it reads. */
export function EditorIslandStage({
  content,
  isShown = true,
}: {
  content: EditorIslandContent;
  isShown?: boolean;
}) {
  return (
    <div className="relative size-full" dir="ltr">
      <div className="absolute inset-x-0" style={{ top: ISLAND.screenGap }}>
        <EditorIsland content={content} isShown={isShown} />
      </div>
    </div>
  );
}

/** Two editor islands open together, as the Captions mode opens them, at
 * the picture's opposite top corners: `end` in the end one, where every
 * island stands, and `start`, the Script island, in the start one, as
 * `EditorIslandPanel` places them. */
export function EditorIslandPairStage({
  end,
  isShown = true,
  start,
}: {
  end: EditorIslandContent;
  start: EditorIslandContent;
  isShown?: boolean;
}) {
  return (
    <div className="relative size-full" dir="ltr">
      <div
        className="absolute inset-x-0 flex items-start justify-between gap-3 px-3"
        style={{ top: ISLAND.screenGap }}
      >
        <EditorIsland content={start} isShown={isShown} />
        <EditorIsland content={end} isShown={isShown} />
      </div>
    </div>
  );
}
