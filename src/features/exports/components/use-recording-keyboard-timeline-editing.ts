// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useMemo } from "react";

import {
  deletedRecordingKeyboardShortcutFragments,
  deletedRecordingKeyboardShortcutIds,
  deleteRecordingKeyboardShortcutFragments,
  keyboardShortcutFragmentId,
  recordingKeyboardShortcutPositions,
} from "../recording-keyboard-timeline-edit";
import { RecordingTimelineEdit } from "../recording-timeline-edit";
import { useExportEditGesture } from "../use-export-edit-history";
import { useExportWindowShortcuts } from "../use-export-window-shortcuts";
import { useRecordingKeyboardTimeline } from "../use-recording-keyboard-timeline";

import { useTimelineItemSelection } from "./timeline-item-selection";

export function useRecordingKeyboardTimelineEditing({
  artifactId,
  edit,
  enabled,
  onChange,
  onSelectionStart,
  sourceDurationMs,
}: {
  artifactId: number;
  edit: RecordingTimelineEdit;
  enabled: boolean;
  sourceDurationMs: number;
  onChange?: (edit: RecordingTimelineEdit) => void;
  onSelectionStart?: () => void;
}) {
  const { appliedEdit, items: allItems } = useRecordingKeyboardTimeline({
    artifactId,
    edit,
    enabled,
    sourceDurationMs,
  });
  const selection = useTimelineItemSelection<string>(onSelectionStart);
  const editGesture = useExportEditGesture();
  // Lane geometry is computed against `appliedEdit`, which trails the live
  // edit by one round trip. Hiding by the union of both keeps deletions
  // instant while an undone deletion stays hidden until the recalculated
  // lane arrives, instead of flashing over its neighbour's stale span.
  const hiddenItemIds = useMemo(
    () =>
      new Set([
        ...deletedRecordingKeyboardShortcutIds(edit),
        ...deletedRecordingKeyboardShortcutIds(appliedEdit),
      ]),
    [appliedEdit, edit],
  );
  const hiddenFragmentIds = useMemo(
    () =>
      new Set(
        [
          ...deletedRecordingKeyboardShortcutFragments(edit),
          ...deletedRecordingKeyboardShortcutFragments(appliedEdit),
        ].map((fragment) =>
          keyboardShortcutFragmentId(fragment.shortcutId, fragment.segmentId),
        ),
      ),
    [appliedEdit, edit],
  );
  const adjustedFragmentIds = useMemo(
    () =>
      new Set(
        recordingKeyboardShortcutPositions(edit).map((position) =>
          keyboardShortcutFragmentId(position.shortcutId, position.segmentId),
        ),
      ),
    [edit],
  );
  const deleteSelected = useCallback(() => {
    if (!onChange || selection.ids.size === 0) return;
    const next = deleteRecordingKeyboardShortcutFragments(edit, selection.ids);
    if (next === edit) return;
    editGesture.beginGesture();
    onChange(next);
    selection.onClear();
    editGesture.endGesture();
  }, [edit, editGesture, onChange, selection]);
  useExportWindowShortcuts({
    onDelete: onChange && selection.ids.size > 0 ? deleteSelected : undefined,
    onDeselect: selection.ids.size > 0 ? selection.onClear : undefined,
  });
  return useMemo(
    () => ({
      adjustedFragmentIds,
      hiddenFragmentIds,
      hiddenItemIds,
      items: allItems,
      selection,
    }),
    [
      adjustedFragmentIds,
      allItems,
      hiddenFragmentIds,
      hiddenItemIds,
      selection,
    ],
  );
}
