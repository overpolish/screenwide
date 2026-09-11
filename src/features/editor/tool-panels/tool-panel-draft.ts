// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { EditorKind } from "../types";

import { ToolPanelPatch, ToolPanelSnapshot } from "./tool-panel-store";

export type ToolPanelDraft = {
  seq: number;
  values: ToolPanelPatch;
  workspace: EditorKind;
};

/** Only pending edits override the mirror; metadata stays live throughout. */
export function resolveToolPanelSnapshot(
  snapshot: ToolPanelSnapshot,
  draft: ToolPanelDraft | null,
  workspace: EditorKind,
): ToolPanelSnapshot {
  return draft?.workspace === workspace &&
    draft.seq > (snapshot.acknowledgedSeq ?? 0)
    ? { ...snapshot, ...draft.values }
    : snapshot;
}
