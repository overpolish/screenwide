// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef } from "react";

import {
  type PopupPanelPart,
  setPopupPanelHitArea,
} from "../../../popup-panel/api";
import {
  sendToolPanelRequest,
  useToolPanelStore,
} from "../../tool-panels/tool-panel-store";
import { usePanelShortcuts } from "../../tool-panels/use-panel-shortcuts";

import {
  isStripIsland,
  type PopupPanelIslandContent,
} from "./editor-island-kind";
import { PlacedIsland } from "./placed-island";
import { useSnapshotIsland } from "./use-snapshot-island";

// The editor's islands, alone in the panel window laid over the editor's
// content area (`use-editor-island-window.ts`). Every island stands at the
// picture's top end corner, whichever tool, mode or choice brought it out,
// so it is always found in one place; a strip item's stands over the item
// at the picture's foot. The Script island, for the Captions and
// Transcript modes, stands at the opposite corner, the top start, in both,
// so moving between the two modes never moves it: only the Captions island
// comes and goes at the end. The window lets clicks through everywhere but
// the islands, whose parts each reports as it changes shape. The Export
// island is the exception: before its export and once done, a press
// anywhere off it puts it away, so while it is out the window takes every
// press, the strip's included, and the press that puts it away reaches
// nothing beneath.

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
  // While it exports, a press off it never stops the export: only Cancel
  // export does.
  const dismissesOnPress = useToolPanelStore((state) => {
    const shown = state.snapshots[content.workspace]?.export;
    return content.island === "export" && shown != null && !shown.progress;
  });
  const partsRef = useRef<Record<"corner" | "script", PopupPanelPart[]>>({
    corner: [],
    script: [],
  });
  const dismissesRef = useRef(dismissesOnPress);
  dismissesRef.current = dismissesOnPress;
  const reportHitArea = () => {
    void setPopupPanelHitArea(
      label,
      dismissesRef.current
        ? [{ height: window.innerHeight, width: window.innerWidth, x: 0, y: 0 }]
        : [...partsRef.current.corner, ...partsRef.current.script],
    );
  };
  const reportOf =
    (island: "corner" | "script") => (parts: PopupPanelPart[]) => {
      partsRef.current[island] = parts;
      reportHitArea();
    };
  // eslint-disable-next-line @eslint-react/exhaustive-deps -- reported again whenever pressing off the island starts or stops putting it away.
  useEffect(reportHitArea, [dismissesOnPress]);

  // Showing something else in this window makes it an ordinary panel again.
  useEffect(
    () => () => {
      void setPopupPanelHitArea(label, null);
    },
    [label],
  );

  return (
    <div className="dark fixed inset-0 text-fg">
      {dismissesOnPress ? (
        <div
          aria-hidden
          className="absolute inset-0"
          onPointerDown={() => {
            sendToolPanelRequest(content.workspace, {
              type: "patch",
              values: { exportPatch: { dismiss: true } },
            });
          }}
        />
      ) : null}
      <PlacedIsland
        at={isStripIsland(content.island) ? content.x : null}
        inset={content.inset}
        onParts={reportOf("corner")}
        shown={corner}
      />
      <PlacedIsland
        corner="start"
        inset={content.inset}
        onParts={reportOf("script")}
        shown={script}
      />
    </div>
  );
}
