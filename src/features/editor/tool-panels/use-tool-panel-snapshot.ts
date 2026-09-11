// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { EditorKind } from "../types";

import { resolveToolPanelSnapshot, ToolPanelDraft } from "./tool-panel-draft";
import {
  DEFAULT_TOOL_PANEL_SNAPSHOT,
  sendToolPanelRequest,
  ToolPanelPatch,
  useToolPanelStore,
} from "./tool-panel-store";

/** Keeps unacknowledged input local without writing it back into the mirror. */
export function useToolPanelSnapshot(workspace: EditorKind) {
  const snapshot =
    useToolPanelStore((state) => state.snapshots[workspace]) ??
    DEFAULT_TOOL_PANEL_SNAPSHOT;
  const [draft, setDraft] = useState<ToolPanelDraft | null>(null);
  useEffect(
    () =>
      useToolPanelStore.subscribe((state) => {
        const acknowledgedSeq =
          state.snapshots[workspace]?.acknowledgedSeq ?? 0;
        setDraft((current) =>
          current?.workspace === workspace && acknowledgedSeq >= current.seq
            ? null
            : current,
        );
      }),
    [workspace],
  );

  return {
    change: (values: ToolPanelPatch) => {
      const seq = sendToolPanelRequest(workspace, { type: "patch", values });
      setDraft({ seq, values, workspace });
    },
    snapshot: resolveToolPanelSnapshot(snapshot, draft, workspace),
  };
}
