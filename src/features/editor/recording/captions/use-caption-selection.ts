// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback } from "react";

import { useEditorWindowShortcuts } from "../../shortcuts/use-editor-window-shortcuts";
import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { useTimelineItemSelection } from "../../timeline/tracks/timeline-item-selection";
import { useEditorEditGesture } from "../../use-editor-edit-history";

import { hideCaptions } from "./recording-captions";

/**
 * The captions chosen in their lane or in the picture, by their lane ids:
 * each caption's id as text. Delete takes them out of the video and the
 * subtitles, as one step of the edit history; the Captions panel's Restore
 * all brings them back.
 */
export function useCaptionSelection({
  edit,
  onChange,
  onSelectionStart,
}: {
  edit: RecordingTimelineEdit;
  onChange?: (edit: RecordingTimelineEdit) => void;
  onSelectionStart?: () => void;
}) {
  const selection = useTimelineItemSelection<string>(onSelectionStart);
  const editGesture = useEditorEditGesture();
  const deleteSelected = useCallback(() => {
    if (!onChange) return;
    const next = hideCaptions(edit, [...selection.ids].map(Number));
    selection.onClear();
    if (next === edit) return;
    editGesture.beginGesture();
    onChange(next);
    editGesture.endGesture();
  }, [edit, editGesture, onChange, selection]);
  const hasSelection = selection.ids.size > 0;
  useEditorWindowShortcuts({
    onDelete: onChange && hasSelection ? deleteSelected : undefined,
    onDeselect: hasSelection ? selection.onClear : undefined,
  });
  return selection;
}
