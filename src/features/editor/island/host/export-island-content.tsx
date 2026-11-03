// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isWindows } from "../../../../lib/platform";
import {
  copyExportFile,
  dragExportFile,
  revealExportFile,
} from "../../export/export-file-api";
import { exportIsland } from "../export/export-island";

import type { ToolPanelPatch } from "../../tools/tool-panel-patch";
import type { ToolPanelSnapshot } from "../../tools/tool-panel-store";
import type { EditorKind } from "../../types";
import type { EditorIslandContent } from "../editor-island";

const report = (action: string) => (cause: unknown) => {
  console.error(`Could not ${action} the exported file`, cause);
};

/** The export the editor publishes, and its asks sent back to it. The
 * finished file is acted on from this window, which a drag starts from;
 * Windows takes neither Copy nor the drag, so the finish offers Reveal
 * alone there. */
export function exportIslandContent({
  change,
  snapshot,
  workspace,
}: {
  change: (values: ToolPanelPatch) => void;
  snapshot: ToolPanelSnapshot;
  workspace: EditorKind;
}): EditorIslandContent | null {
  if (!snapshot.export) return null;
  const isMac = !isWindows();
  return exportIsland({
    export: snapshot.export,
    finish: {
      onCopy: isMac
        ? () => {
            copyExportFile(workspace).catch(report("copy"));
          }
        : undefined,
      onDragOut: isMac
        ? () => {
            dragExportFile(workspace).catch(report("drag"));
          }
        : undefined,
      onReveal: () => {
        revealExportFile(workspace).catch(report("reveal"));
      },
    },
    onPatch: (exportPatch) => {
      change({ exportPatch });
    },
  });
}
