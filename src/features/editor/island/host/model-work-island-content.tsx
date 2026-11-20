// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { modelWorkIsland } from "../model-work/model-work-island";

import type { ToolPanelPatch } from "../../tools/tool-panel-patch";
import type { ToolPanelSnapshot } from "../../tools/tool-panel-store";
import type { EditorIslandContent } from "../editor-island";

// The models' work as the editor publishes it, each Cancel sent back to the
// editor, which stops the work and turns its tool off.

export function modelWorkIslandContent({
  change,
  snapshot,
}: {
  change: (values: ToolPanelPatch) => void;
  snapshot: ToolPanelSnapshot;
}): EditorIslandContent | null {
  return modelWorkIsland({
    jobs: snapshot.modelWork ?? [],
    onCancel: (cancelModelWork) => {
      change({ cancelModelWork });
    },
  });
}
