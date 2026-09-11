// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SelectionPlacementPatch } from "../selection-placement";
import { EditorKind } from "../types";

import { ToolPanelPatch, ToolPanelSnapshot } from "./tool-panel-store";

export type ToolPanelDraft = {
  seq: number;
  values: ToolPanelPatch;
  workspace: EditorKind;
};

/** Only the fields the panel is holding override the published placement, so a
 * drag in the preview keeps moving the rest of them under an open field. */
const placedSelection = (
  selection: ToolPanelSnapshot["selection"],
  placement: SelectionPlacementPatch,
) => {
  if (!selection) return selection;
  const next = { ...selection };
  for (const key of ["height", "width", "x", "y"] as const) {
    const value = placement[key];
    if (value !== undefined) next[key] = value;
  }
  return next;
};

/** Only pending edits override the mirror; metadata stays live throughout. */
export function resolveToolPanelSnapshot(
  snapshot: ToolPanelSnapshot,
  draft: ToolPanelDraft | null,
  workspace: EditorKind,
): ToolPanelSnapshot {
  if (
    draft?.workspace !== workspace ||
    draft.seq <= (snapshot.acknowledgedSeq ?? 0)
  )
    return snapshot;
  // A reset is an action rather than a value: nothing of it is shown locally,
  // and the editor's answer arrives as the next published placement.
  const { resetSelection: _reset, selectionOutput, ...values } = draft.values;
  const resolved = { ...snapshot, ...values };
  return selectionOutput
    ? {
        ...resolved,
        selection: placedSelection(resolved.selection, selectionOutput),
      }
    : resolved;
}
