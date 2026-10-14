// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEditorEditGesture } from "../../use-editor-edit-history";
import { RecordingTimelineEdit } from "../editing/recording-timeline-edit";

import {
  deletedRecordingKeyboardShortcutFragments,
  deletedRecordingKeyboardShortcutIds,
  restoreRecordingKeyboardShortcuts,
} from "./recording-keyboard-timeline-edit";

export function useRestoreRecordingKeyboardShortcuts(
  edit: RecordingTimelineEdit | null | undefined,
  onChange?: (edit: RecordingTimelineEdit) => void,
) {
  const editGesture = useEditorEditGesture();
  const canRestore = Boolean(
    edit &&
    onChange &&
    (deletedRecordingKeyboardShortcutIds(edit).length > 0 ||
      deletedRecordingKeyboardShortcutFragments(edit).length > 0),
  );
  const restore = () => {
    if (!edit || !onChange) return;
    const restored = restoreRecordingKeyboardShortcuts(edit);
    if (restored === edit) return;
    editGesture.beginGesture();
    onChange(restored);
    editGesture.endGesture();
  };
  return { canRestore, restore };
}
