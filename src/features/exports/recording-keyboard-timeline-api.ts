// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import {
  deletedRecordingKeyboardShortcutIds,
  deletedRecordingKeyboardShortcutRanges,
  DeletedKeyboardShortcutRange,
  KeyboardShortcutPositionRange,
  recordingKeyboardShortcutPositionRanges,
} from "./recording-keyboard-timeline-edit";
import { RecordingTimelineEdit } from "./recording-timeline-edit";
import {
  recordingTimelinePlaybackRanges,
  RecordingTimelinePlaybackRange,
} from "./recording-timeline-playback";
import { RecordingKeyboardTimelineItem } from "./types";

export const getRecordingKeyboardTimeline = (
  artifactId: number,
  deletions: RecordingPreviewKeyboardDeletions,
) =>
  invoke<RecordingKeyboardTimelineItem[]>("get_recording_keyboard_timeline", {
    artifactId,
    playbackRanges: deletions.playbackRanges,
    shortcutIds: deletions.deletedKeyboardShortcutIds,
    shortcutPositions: deletions.keyboardShortcutPositions,
    shortcutRanges: deletions.deletedKeyboardShortcutRanges,
  });

export type RecordingPreviewKeyboardDeletions = {
  deletedKeyboardShortcutIds: number[];
  deletedKeyboardShortcutRanges: DeletedKeyboardShortcutRange[];
  keyboardShortcutPositions: KeyboardShortcutPositionRange[];
  playbackRanges: RecordingTimelinePlaybackRange[];
};

export const recordingPreviewKeyboardDeletions = (
  edit: RecordingTimelineEdit | null | undefined,
  sourceDurationMs: number,
): RecordingPreviewKeyboardDeletions => ({
  deletedKeyboardShortcutIds: deletedRecordingKeyboardShortcutIds(edit),
  deletedKeyboardShortcutRanges: deletedRecordingKeyboardShortcutRanges(
    edit,
    sourceDurationMs,
  ),
  keyboardShortcutPositions: recordingKeyboardShortcutPositionRanges(
    edit,
    sourceDurationMs,
  ),
  playbackRanges: recordingTimelinePlaybackRanges(edit, sourceDurationMs).map(
    (range) => ({
      ...range,
      sourceEndMs: Math.round(range.sourceEndMs),
      sourceStartMs: Math.round(range.sourceStartMs),
    }),
  ),
});

export const setRecordingPreviewDeletedKeyboardShortcuts = (
  deletions: RecordingPreviewKeyboardDeletions,
  sessionId: number,
) =>
  invoke<null>("set_recording_preview_deleted_keyboard_shortcuts", {
    playbackRanges: deletions.playbackRanges,
    sessionId,
    shortcutIds: deletions.deletedKeyboardShortcutIds,
    shortcutPositions: deletions.keyboardShortcutPositions,
    shortcutRanges: deletions.deletedKeyboardShortcutRanges,
  });
