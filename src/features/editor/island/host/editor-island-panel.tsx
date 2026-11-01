// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef } from "react";

import {
  type PopupPanelPart,
  setPopupPanelHitArea,
} from "../../../popup-panel/api";
import { usePanelShortcuts } from "../../tool-panels/use-panel-shortcuts";

import {
  isStripIsland,
  type PopupPanelIslandContent,
} from "./editor-island-kind";
import { PlacedIsland } from "./placed-island";
import { useSnapshotIsland } from "./use-snapshot-island";

// The editor's islands, alone in the panel window laid over the picture
// (`use-editor-island-window.ts`). Every island stands at the picture's top
// end corner, whichever tool, mode or choice brought it out, so it is
// always found in one place; a strip item's stands over the item at the
// picture's foot. The Script island, for the Captions and Transcript
// modes, stands at the opposite corner, the top start, in both, so moving
// between the two modes never moves it: only the Captions island comes and
// goes at the end. The window lets clicks through everywhere but the
// islands, whose parts each reports as it changes shape.

export function EditorIslandPanel({
  content,
}: {
  content: PopupPanelIslandContent;
}) {
  usePanelShortcuts(content.workspace);
  const { corner, script } = useSnapshotIsland(
    content.island,
    content.workspace,
  );
  const label = getCurrentWindow().label;
  const partsRef = useRef<Record<"corner" | "script", PopupPanelPart[]>>({
    corner: [],
    script: [],
  });
  const reportOf =
    (island: "corner" | "script") => (parts: PopupPanelPart[]) => {
      partsRef.current[island] = parts;
      void setPopupPanelHitArea(label, [
        ...partsRef.current.corner,
        ...partsRef.current.script,
      ]);
    };

  // Showing something else in this window makes it an ordinary panel again.
  useEffect(
    () => () => {
      void setPopupPanelHitArea(label, null);
    },
    [label],
  );

  return (
    <div className="dark fixed inset-0 text-fg">
      <PlacedIsland
        at={isStripIsland(content.island) ? content.x : null}
        onParts={reportOf("corner")}
        shown={corner}
      />
      <PlacedIsland
        corner="start"
        onParts={reportOf("script")}
        shown={script}
      />
    </div>
  );
}
