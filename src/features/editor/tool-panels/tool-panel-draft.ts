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
  if (!selection || selection.kind === "audio" || selection.kind === "shortcut")
    return selection;
  const next = { ...selection };
  for (const key of ["height", "width", "x", "y"] as const) {
    const value = placement[key];
    if (value !== undefined) next[key] = value;
  }
  return next;
};

/** The same rule for the shortcut: a size being typed leaves the position the
 * drag on the picture is still moving alone. */
const placedShortcut = (
  selection: ToolPanelSnapshot["selection"],
  placement: NonNullable<ToolPanelPatch["shortcutPlacement"]>,
) => {
  if (selection?.kind !== "shortcut") return selection;
  const next = { ...selection };
  for (const key of [
    "positionXPercent",
    "positionYPercent",
    "sizePercent",
  ] as const) {
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
    annotationAngle,
    annotationAnimated,
    annotationStyle,
    applyShortcutToAll: _applyShortcutToAll,
    audioVolume,
    bakeCamera,
    cropSize,
    frameRadius,
    frameSize,
    keyboardEffects,
    recenterSelection: _recenterSelection,
    removeAnnotationColor: _removeAnnotationColor,
    removePreset: _removePreset,
    resetAllShortcuts: _resetAllShortcuts,
    resetCrop: _resetCrop,
    resetFrame: _resetFrame,
    resetKeyboardPosition: _resetKeyboardPosition,
    resetSelection: _resetSelection,
    resetShortcut: _resetShortcut,
    restoreShortcuts: _restoreShortcuts,
    saveAnnotationColor: _saveAnnotationColor,
    savePreset: _savePreset,
    selectionDropShadow,
    selectionInset,
    selectionOutput,
    selectionRadius,
    shortcutPlacement,
    ...values
  } = draft.values;
  const resolved = {
    ...snapshot,
    ...values,
    // A shortcut setting is sent one field at a time, so the rest of the group
    // keeps whatever the editor last published for it.
    ...(keyboardEffects
      ? { keyboardEffects: { ...snapshot.keyboardEffects, ...keyboardEffects } }
      : {}),
  };
  // A dragged inset holds its own value until the editor acknowledges it, so
  // the knob stays under the pointer rather than snapping back a frame.
  const placed =
    resolved.selection?.kind === "shortcut" ||
    resolved.selection?.kind === "audio"
      ? null
      : resolved.selection;
  const held =
    placed && selectionInset !== undefined
      ? { ...placed, inset: selectionInset }
      : placed;
  const rounded =
    held && selectionRadius !== undefined
      ? { ...held, radius: selectionRadius }
      : held;
  const layer =
    rounded && selectionDropShadow !== undefined
      ? { ...rounded, dropShadow: selectionDropShadow }
      : rounded;
  // The bake switch is the same idea: the flip is shown at once, and the
  // placement the editor carries across arrives with its answer.
  const baked =
    layer?.kind === "camera" && bakeCamera !== undefined
      ? { ...layer, isBaked: bakeCamera }
      : layer;
  // A dragged volume is held the same way: the knob stays where it was let go
  // of until the editor answers with the level it committed.
  const heard =
    resolved.selection?.kind === "audio" && audioVolume !== undefined
      ? { ...resolved.selection, decibels: audioVolume }
      : null;
  const selection = baked ?? heard ?? resolved.selection;
  // A colour being dragged in the system panel shows at once: the mirror
  // catches up an edit later, and the swatch must not blink back meanwhile.
  // The Animate switch and the aim are held the same way, so neither flicks
  // back to the mirror's value between the edit and the commit.
  const aimed =
    resolved.annotation && annotationAngle !== undefined
      ? { ...resolved.annotation, angle: annotationAngle }
      : resolved.annotation;
  const switched =
    aimed && annotationAnimated !== undefined
      ? { ...aimed, animated: annotationAnimated }
      : aimed;
  const annotation =
    switched && annotationStyle
      ? { ...switched, style: { ...switched.style, ...annotationStyle } }
      : switched;
  return {
    ...resolved,
    annotation,
    selection,
    ...(cropSize ? { crop: sized(resolved.crop, cropSize) } : {}),
    ...(frameSize ? { frame: sized(resolved.frame, frameSize) } : {}),
    ...(frameRadius !== undefined && resolved.frame
      ? { frame: { ...resolved.frame, radius: frameRadius } }
      : {}),
    ...(selectionOutput
      ? { selection: placedSelection(selection, selectionOutput) }
      : {}),
    ...(shortcutPlacement
      ? { selection: placedShortcut(selection, shortcutPlacement) }
      : {}),
  };
}
