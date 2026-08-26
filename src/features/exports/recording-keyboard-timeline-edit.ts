// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingTimelineEdit } from "./recording-timeline-edit";

export type DeletedKeyboardShortcutFragment = {
  segmentId: number;
  shortcutId: number;
};

export type DeletedKeyboardShortcutRange = {
  endMs: number;
  shortcutId: number;
  startMs: number;
};

export type KeyboardShortcutPosition = {
  centerX: number;
  centerY: number;
  segmentId: number;
  shortcutId: number;
  sizePercent?: number;
};

export type KeyboardShortcutPositionRange = Omit<
  KeyboardShortcutPosition,
  "segmentId"
> & {
  endMs: number;
  startMs: number;
};

type RecordingKeyboardTimelineEdit = RecordingTimelineEdit & {
  deletedKeyboardShortcutFragments?: DeletedKeyboardShortcutFragment[];
  deletedKeyboardShortcutIds?: number[];
  keyboardShortcutPositions?: KeyboardShortcutPosition[];
};

const EMPTY_DELETED_SHORTCUT_FRAGMENTS: DeletedKeyboardShortcutFragment[] = [];
const EMPTY_DELETED_SHORTCUT_IDS: number[] = [];
const EMPTY_SHORTCUT_POSITIONS: KeyboardShortcutPosition[] = [];

const keyboardEdit = (
  edit: RecordingTimelineEdit | null | undefined,
): RecordingKeyboardTimelineEdit | null | undefined => edit;

export const deletedRecordingKeyboardShortcutIds = (
  edit: RecordingTimelineEdit | null | undefined,
) =>
  keyboardEdit(edit)?.deletedKeyboardShortcutIds ?? EMPTY_DELETED_SHORTCUT_IDS;

export const deletedRecordingKeyboardShortcutFragments = (
  edit: RecordingTimelineEdit | null | undefined,
) =>
  keyboardEdit(edit)?.deletedKeyboardShortcutFragments ??
  EMPTY_DELETED_SHORTCUT_FRAGMENTS;

export const recordingKeyboardShortcutPositions = (
  edit: RecordingTimelineEdit | null | undefined,
) => keyboardEdit(edit)?.keyboardShortcutPositions ?? EMPTY_SHORTCUT_POSITIONS;

export const keyboardShortcutFragmentId = (
  shortcutId: number,
  segmentId: number,
) => `${shortcutId.toString()}:${segmentId.toString()}`;

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
  const next: RecordingKeyboardTimelineEdit = {
    ...edit,
    deletedKeyboardShortcutFragments: [...fragments.values()].sort(
      (a, b) => a.shortcutId - b.shortcutId || a.segmentId - b.segmentId,
    ),
  };
  return next;
}

export function moveRecordingKeyboardShortcutFragments({
  bounds,
  delta,
  edit,
  fragmentIds,
  leaderCenter,
}: {
  bounds: { height: number; width: number };
  delta: { x: number; y: number };
  edit: RecordingTimelineEdit;
  fragmentIds: Iterable<string>;
  leaderCenter: { x: number; y: number };
}): RecordingTimelineEdit {
  const positions = new Map(
    recordingKeyboardShortcutPositions(edit).map((position) => [
      keyboardShortcutFragmentId(position.shortcutId, position.segmentId),
      position,
    ]),
  );
  let changed = false;
  for (const fragmentId of fragmentIds) {
    const fragment = parseKeyboardShortcutFragmentId(fragmentId);
    if (!fragment) continue;
    const centerX = Math.max(
      bounds.width / 2,
      Math.min(1 - bounds.width / 2, leaderCenter.x + delta.x),
    );
    const centerY = Math.max(
      bounds.height / 2,
      Math.min(1 - bounds.height / 2, leaderCenter.y + delta.y),
    );
    positions.set(fragmentId, {
      ...positions.get(fragmentId),
      ...fragment,
      centerX,
      centerY,
    });
    changed = true;
  }
  if (!changed) return edit;
  const next: RecordingKeyboardTimelineEdit = {
    ...edit,
    keyboardShortcutPositions: [...positions.values()].sort(
      (a, b) => a.shortcutId - b.shortcutId || a.segmentId - b.segmentId,
    ),
  };
  return next;
}

export function resizeRecordingKeyboardShortcutFragments({
  center,
  edit,
  fragmentIds,
  maximumSizePercent,
  minimumSizePercent,
  sizePercent,
}: {
  center: { x: number; y: number };
  edit: RecordingTimelineEdit;
  fragmentIds: Iterable<string>;
  sizePercent: number;
  maximumSizePercent?: number;
  minimumSizePercent?: number;
}): RecordingTimelineEdit {
  const positions = new Map(
    recordingKeyboardShortcutPositions(edit).map((position) => [
      keyboardShortcutFragmentId(position.shortcutId, position.segmentId),
      position,
    ]),
  );
  let changed = false;
  for (const fragmentId of fragmentIds) {
    const fragment = parseKeyboardShortcutFragmentId(fragmentId);
    if (!fragment) continue;
    positions.set(fragmentId, {
      ...positions.get(fragmentId),
      ...fragment,
      centerX: center.x,
      centerY: center.y,
      sizePercent: Math.max(
        minimumSizePercent ?? 5,
        Math.min(maximumSizePercent ?? 500, sizePercent),
      ),
    });
    changed = true;
  }
  if (!changed) return edit;
  return {
    ...edit,
    keyboardShortcutPositions: [...positions.values()].sort(
      (a, b) => a.shortcutId - b.shortcutId || a.segmentId - b.segmentId,
    ),
  } as RecordingKeyboardTimelineEdit;
}

export function resetAllRecordingKeyboardShortcutPositions(
  edit: RecordingTimelineEdit,
): RecordingTimelineEdit {
  const keyboard = edit as RecordingKeyboardTimelineEdit;
  if (!keyboard.keyboardShortcutPositions?.length) return edit;
  const { keyboardShortcutPositions: _positions, ...rest } = keyboard;
  return rest;
}

export function resetRecordingKeyboardShortcutPositions(
  edit: RecordingTimelineEdit,
  fragmentIds: Iterable<string>,
): RecordingTimelineEdit {
  const reset = new Set(fragmentIds);
  const positions = recordingKeyboardShortcutPositions(edit).filter(
    (position) =>
      !reset.has(
        keyboardShortcutFragmentId(position.shortcutId, position.segmentId),
      ),
  );
  if (positions.length === recordingKeyboardShortcutPositions(edit).length)
    return edit;
  const keyboard = edit as RecordingKeyboardTimelineEdit;
  if (positions.length > 0) {
    const next: RecordingKeyboardTimelineEdit = {
      ...keyboard,
      keyboardShortcutPositions: positions,
    };
    return next;
  }
  const { keyboardShortcutPositions: _positions, ...rest } = keyboard;
  return rest;
}

export function restoreRecordingKeyboardShortcuts(
  edit: RecordingTimelineEdit,
): RecordingTimelineEdit {
  const keyboard = edit as RecordingKeyboardTimelineEdit;
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

export function recordingKeyboardShortcutPositionRanges(
  edit: RecordingTimelineEdit | null | undefined,
  sourceDurationMs: number,
): KeyboardShortcutPositionRange[] {
  if (!edit || sourceDurationMs <= 0) return [];
  const segments = new Map(
    edit.segments.map((segment) => [segment.id, segment]),
  );
  return recordingKeyboardShortcutPositions(edit).flatMap(
    ({ centerX, centerY, segmentId, shortcutId, sizePercent }) => {
      const segment = segments.get(segmentId);
      return segment
        ? [
            {
              centerX,
              centerY,
              endMs: Math.round(segment.sourceEnd * sourceDurationMs),
              ...(sizePercent === undefined ? {} : { sizePercent }),
              shortcutId,
              startMs: Math.round(segment.sourceStart * sourceDurationMs),
            },
          ]
        : [];
    },
  );
}
