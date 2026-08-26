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
}: {
  artifactId: number;
  edit: RecordingTimelineEdit;
  enabled: boolean;
  onChange?: (edit: RecordingTimelineEdit) => void;
  onSelectionStart?: () => void;
}) {
  const allItems = useRecordingKeyboardTimeline(artifactId, enabled);
  const selection = useTimelineItemSelection<string>(onSelectionStart);
  const editGesture = useExportEditGesture();
  const hiddenItemIds = useMemo(
    () => new Set(deletedRecordingKeyboardShortcutIds(edit)),
    [edit],
  );
  const hiddenFragmentIds = useMemo(
    () =>
      new Set(
        deletedRecordingKeyboardShortcutFragments(edit).map((fragment) =>
          keyboardShortcutFragmentId(fragment.shortcutId, fragment.segmentId),
        ),
      ),
    [edit],
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
