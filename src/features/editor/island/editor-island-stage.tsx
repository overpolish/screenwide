// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ISLAND } from "../../../primitives/island/island-geometry";

import { EditorIsland, type EditorIslandContent } from "./editor-island";

/** An editor island as its window shows it, hung at the top of the canvas
 * as it hangs under what opened it. Laid out left to right, like the strip
 * and the picture it reads. */
export function EditorIslandStage({
  content,
}: {
  content: EditorIslandContent;
}) {
  return (
    <div className="relative size-full" dir="ltr">
      <div className="absolute inset-x-0" style={{ top: ISLAND.screenGap }}>
        <EditorIsland content={content} />
      </div>
    </div>
  );
}
