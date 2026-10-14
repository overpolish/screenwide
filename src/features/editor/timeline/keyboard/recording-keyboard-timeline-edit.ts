// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingTimelineEdit } from "../editing/recording-timeline-edit";

import type { DeletedKeyboardShortcutFragment } from "../../../../bindings/DeletedKeyboardShortcutFragment";
import type { DeletedKeyboardShortcutRange } from "../../../../bindings/DeletedKeyboardShortcutRange";

export type { DeletedKeyboardShortcutRange };

const EMPTY_DELETED_SHORTCUT_FRAGMENTS: DeletedKeyboardShortcutFragment[] = [];
const EMPTY_DELETED_SHORTCUT_IDS: number[] = [];

const keyboardEdit = (
  edit: RecordingTimelineEdit | null | undefined,
): RecordingTimelineEdit | null | undefined => edit;

export const deletedRecordingKeyboardShortcutIds = (
  edit: RecordingTimelineEdit | null | undefined,
) =>
  keyboardEdit(edit)?.deletedKeyboardShortcutIds ?? EMPTY_DELETED_SHORTCUT_IDS;

export const deletedRecordingKeyboardShortcutFragments = (
  edit: RecordingTimelineEdit | null | undefined,
) =>
  keyboardEdit(edit)?.deletedKeyboardShortcutFragments ??
  EMPTY_DELETED_SHORTCUT_FRAGMENTS;

export const keyboardShortcutFragmentId = (
  shortcutId: number,
  segmentId: number,
) => `${shortcutId.toString()}:${segmentId.toString()}`;

/** Copies segment-keyed shortcut edits onto newly split descendants. */
export function inheritRecordingKeyboardSegmentEdits(
  edit: RecordingTimelineEdit,
  next: RecordingTimelineEdit,
  descendantIds: ReadonlyMap<number, readonly number[]>,
): RecordingTimelineEdit {
  const inheritedDeleted = deletedRecordingKeyboardShortcutFragments(
    edit,
  ).flatMap((fragment) =>
    (descendantIds.get(fragment.segmentId) ?? []).map((segmentId) => ({
      ...fragment,
      segmentId,
    })),
  );
  if (inheritedDeleted.length === 0) return next;
  return {
    ...next,
    deletedKeyboardShortcutFragments: [
      ...deletedRecordingKeyboardShortcutFragments(edit),
      ...inheritedDeleted,
    ].sort((a, b) => a.shortcutId - b.shortcutId || a.segmentId - b.segmentId),
  };
}

const parseKeyboardShortcutFragmentId = (fragmentId: string) => {
  const parts = fragmentId.split(":");
  if (parts.length !== 2) return null;
  const [shortcutId, segmentId] = parts.map(Number);
  return Number.isInteger(shortcutId) &&
    shortcutId >= 0 &&
    Number.isInteger(segmentId) &&
    segmentId >= 0
    ? { segmentId, shortcutId }
    : null;
};

export function deleteRecordingKeyboardShortcutFragments(
  edit: RecordingTimelineEdit,
  fragmentIds: Iterable<string>,
): RecordingTimelineEdit {
  const fragments = new Map(
    deletedRecordingKeyboardShortcutFragments(edit).map((fragment) => [
      keyboardShortcutFragmentId(fragment.shortcutId, fragment.segmentId),
      fragment,
    ]),
  );
  const previousSize = fragments.size;
  for (const fragmentId of fragmentIds) {
    const fragment = parseKeyboardShortcutFragmentId(fragmentId);
    if (fragment) fragments.set(fragmentId, fragment);
  }
  if (fragments.size === previousSize) return edit;
  const next: RecordingTimelineEdit = {
    ...edit,
    deletedKeyboardShortcutFragments: [...fragments.values()].sort(
      (a, b) => a.shortcutId - b.shortcutId || a.segmentId - b.segmentId,
    ),
  };
  return next;
}

export function restoreRecordingKeyboardShortcuts(
  edit: RecordingTimelineEdit,
): RecordingTimelineEdit {
  const keyboard = edit;
  if (
    !keyboard.deletedKeyboardShortcutIds?.length &&
    !keyboard.deletedKeyboardShortcutFragments?.length
  )
    return edit;
  const {
    deletedKeyboardShortcutFragments: _fragments,
    deletedKeyboardShortcutIds: _ids,
    ...rest
  } = keyboard;
  return rest;
}

export function deletedRecordingKeyboardShortcutRanges(
  edit: RecordingTimelineEdit | null | undefined,
  sourceDurationMs: number,
): DeletedKeyboardShortcutRange[] {
  if (!edit || sourceDurationMs <= 0) return [];
  const segments = new Map(
    edit.segments.map((segment) => [segment.id, segment]),
  );
  return deletedRecordingKeyboardShortcutFragments(edit).flatMap(
    ({ segmentId, shortcutId }) => {
      const segment = segments.get(segmentId);
      return segment
        ? [
            {
              endMs: Math.round(segment.sourceEnd * sourceDurationMs),
              shortcutId,
              startMs: Math.round(segment.sourceStart * sourceDurationMs),
            },
          ]
        : [];
    },
  );
}
