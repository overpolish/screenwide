// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { lullIsland } from "../../../../features/editor/island/lull/lull-island";
import { ownLength, speedOf } from "../model/speed";
import { type Editor } from "../model/use-editor";

import { type IslandContent } from "./island-parts";

/** A pause on the exploration's strip, as the editor's lull island holds
 * it, read from the exploration's timing. */
export function lullContent(editor: Editor, id: string): IslandContent | null {
  const pause = editor.map.pauses.find((item) => item.id === id);
  if (!pause) return null;
  const setting = editor.settings[id] as Editor["settings"][string] | undefined;
  // A lull retimed back to its recorded speed plays as one kept as
  // recorded, and the island shows it so.
  const speed =
    setting?.kind === "own"
      ? speedOf(
          pause.end - pause.start,
          ownLength(pause, setting.seconds, editor.recording.look),
        )
      : null;
  const content = lullIsland({
    id,
    onCut: editor.remove,
    onPlayChange: (play) => {
      editor.setPause(id, play === "keep" ? { kind: "keep" } : null);
    },
    onTrimToBreath: () => {
      editor.trimToBreath(id);
    },
    play:
      speed !== null
        ? { kind: "retimed", speed }
        : { kind: setting ? "keep" : "squeeze" },
  });
  return {
    actions: content.actions,
    from: { kind: "strip" },
    key: content.key,
    section: { key: content.key, node: content.section },
    title: content.title,
  };
}
