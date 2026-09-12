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

/** The same rule for the canvas and the crop: a width being typed does not pin
 * the height a drag on the picture is still moving. */
const sized = <Value extends { height: number; width: number }>(
  value: Value | null,
  size: { height?: number; width?: number },
) => {
  if (!value) return value;
  const next = { ...value };
  for (const key of ["height", "width"] as const) {
    const typed = size[key];
    if (typed !== undefined) next[key] = typed;
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
  const {
    cropSize,
    frameSize,
    recenterSelection: _recenterSelection,
    removePreset: _removePreset,
    resetCrop: _resetCrop,
    resetFrame: _resetFrame,
    resetSelection: _resetSelection,
    savePreset: _savePreset,
    selectionDropShadow,
    selectionInset,
    selectionOutput,
    selectionRadius,
    ...values
  } = draft.values;
  const resolved = { ...snapshot, ...values };
  // A dragged inset holds its own value until the editor acknowledges it, so
  // the knob stays under the pointer rather than snapping back a frame.
  const held =
    resolved.selection && selectionInset !== undefined
      ? { ...resolved.selection, inset: selectionInset }
      : resolved.selection;
  const rounded =
    held && selectionRadius !== undefined
      ? { ...held, radius: selectionRadius }
      : held;
  const selection =
    rounded && selectionDropShadow !== undefined
      ? { ...rounded, dropShadow: selectionDropShadow }
      : rounded;
  return {
    ...resolved,
    selection,
    ...(cropSize ? { crop: sized(resolved.crop, cropSize) } : {}),
    ...(frameSize ? { frame: sized(resolved.frame, frameSize) } : {}),
    ...(selectionOutput
      ? { selection: placedSelection(selection, selectionOutput) }
      : {}),
  };
}
