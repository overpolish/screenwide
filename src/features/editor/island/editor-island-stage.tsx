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

/** Two editor islands open together, one at each side of the canvas with
 * the picture between them, as a mode that needs both opens them: the
 * first at the start, the second at the end, each hung at the top. */
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
        className="absolute inset-x-0 flex items-start justify-between px-6"
        style={{ top: ISLAND.screenGap }}
      >
        <EditorIsland content={start} isShown={isShown} />
        <EditorIsland content={end} isShown={isShown} />
      </div>
    </div>
  );
}
