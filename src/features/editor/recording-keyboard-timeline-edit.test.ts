// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { cutKeyboardTimeline } from "./recording-keyboard-timeline-cut";
import {
  deletedRecordingKeyboardShortcutRanges,
  deleteRecordingKeyboardShortcutFragments,
  moveRecordingKeyboardShortcutFragments,
  recordingKeyboardShortcutPositionRanges,
  recordingKeyboardShortcutPositions,
  resetAllRecordingKeyboardShortcutPositions,
  resetRecordingKeyboardShortcutPositions,
  resizeRecordingKeyboardShortcutFragments,
  restoreRecordingKeyboardShortcuts,
} from "./recording-keyboard-timeline-edit";
import { createRecordingTimelineEdit } from "./recording-timeline-edit";

describe("recording keyboard shortcut deletion", () => {
  it("copies fragment deletion and transforms into a bladed segment", () => {
    const edited = {
      ...createRecordingTimelineEdit(7),
      deletedKeyboardShortcutFragments: [{ segmentId: 0, shortcutId: 8 }],
      keyboardShortcutPositions: [
        {
          centerX: 0.4,
          centerY: 0.6,
          segmentId: 0,
          shortcutId: 4,
          sizePercent: 135,
        },
      ],
    };
    const cut = cutKeyboardTimeline(edited, 0.5);

    expect(cut).toMatchObject({
      deletedKeyboardShortcutFragments: [
        { segmentId: 0, shortcutId: 8 },
        { segmentId: 1, shortcutId: 8 },
      ],
      keyboardShortcutPositions: [
        {
          centerX: 0.4,
          centerY: 0.6,
          segmentId: 0,
          shortcutId: 4,
          sizePercent: 135,
        },
        {
          centerX: 0.4,
          centerY: 0.6,
          segmentId: 1,
          shortcutId: 4,
          sizePercent: 135,
        },
      ],
    });
  });

  it("adds stable fragment ids once in deterministic order", () => {
    const edit = createRecordingTimelineEdit(7);
    const deleted = deleteRecordingKeyboardShortcutFragments(edit, [
      "4:0",
      "2:0",
      "4:0",
    ]);
    expect(deleted).toMatchObject({
      deletedKeyboardShortcutFragments: [
        { segmentId: 0, shortcutId: 2 },
        { segmentId: 0, shortcutId: 4 },
      ],
    });
    expect(deleteRecordingKeyboardShortcutFragments(deleted, ["2:0"])).toBe(
      deleted,
    );
  });

  it("maps a fragment to its segment source-time range", () => {
    const edit = deleteRecordingKeyboardShortcutFragments(
      {
        artifactId: 7,
        nextSegmentId: 2,
        segments: [
          { id: 0, sourceEnd: 0.4, sourceStart: 0 },
          { id: 1, sourceEnd: 1, sourceStart: 0.6 },
        ],
      },
      ["4:1"],
    );
    expect(deletedRecordingKeyboardShortcutRanges(edit, 10_000)).toEqual([
      { endMs: 10_000, shortcutId: 4, startMs: 6_000 },
    ]);
  });

  it("restores deleted shortcuts without changing timeline segments", () => {
    const edit = deleteRecordingKeyboardShortcutFragments(
      {
        artifactId: 7,
        nextSegmentId: 3,
        segments: [
          { id: 0, sourceEnd: 0.4, sourceStart: 0 },
          { id: 2, sourceEnd: 1, sourceStart: 0.6 },
        ],
      },
      ["4:2"],
    );
    const restored = restoreRecordingKeyboardShortcuts(edit);
    expect(restored).toEqual({
      artifactId: 7,
      nextSegmentId: 3,
      segments: [
        { id: 0, sourceEnd: 0.4, sourceStart: 0 },
        { id: 2, sourceEnd: 1, sourceStart: 0.6 },
      ],
    });
    expect(restoreRecordingKeyboardShortcuts(restored)).toBe(restored);
  });
});

describe("recording keyboard shortcut positions", () => {
  it("moves only selected fragments from the inherited default", () => {
    const edit = createRecordingTimelineEdit(7);
    const moved = moveRecordingKeyboardShortcutFragments({
      bounds: { height: 0.1, width: 0.2 },
      delta: { x: 0.1, y: -0.2 },
      edit,
      fragmentIds: ["4:0", "6:0"],
      leaderCenter: { x: 0.5, y: 0.9 },
    });
    expect(moved).toMatchObject({
      keyboardShortcutPositions: [
        { centerX: 0.6, centerY: 0.7, segmentId: 0, shortcutId: 4 },
        { centerX: 0.6, centerY: 0.7, segmentId: 0, shortcutId: 6 },
      ],
    });
    expect(recordingKeyboardShortcutPositionRanges(moved, 10_000)).toEqual([
      {
        centerX: 0.6,
        centerY: 0.7,
        endMs: 10_000,
        shortcutId: 4,
        startMs: 0,
      },
      {
        centerX: 0.6,
        centerY: 0.7,
        endMs: 10_000,
        shortcutId: 6,
        startMs: 0,
      },
    ]);
  });

  it("resets selected overrides without restoring deleted shortcuts", () => {
    const deleted = deleteRecordingKeyboardShortcutFragments(
      createRecordingTimelineEdit(7),
      ["8:0"],
    );
    const moved = moveRecordingKeyboardShortcutFragments({
      bounds: { height: 0.1, width: 0.2 },
      delta: { x: 0.1, y: 0 },
      edit: deleted,
      fragmentIds: ["4:0", "6:0"],
      leaderCenter: { x: 0.5, y: 0.9 },
    });
    expect(
      resetRecordingKeyboardShortcutPositions(moved, ["4:0"]),
    ).toMatchObject({
      deletedKeyboardShortcutFragments: [{ segmentId: 0, shortcutId: 8 }],
      keyboardShortcutPositions: [
        { centerX: 0.6, centerY: 0.9, segmentId: 0, shortcutId: 6 },
      ],
    });
  });

  it("moves every selected fragment to the leader position", () => {
    const movedOnce = moveRecordingKeyboardShortcutFragments({
      bounds: { height: 0.1, width: 0.2 },
      delta: { x: -0.2, y: -0.1 },
      edit: createRecordingTimelineEdit(7),
      fragmentIds: ["4:0"],
      leaderCenter: { x: 0.5, y: 0.9 },
    });
    const movedTogether = moveRecordingKeyboardShortcutFragments({
      bounds: { height: 0.1, width: 0.2 },
      delta: { x: 0.1, y: -0.2 },
      edit: movedOnce,
      fragmentIds: ["4:0", "6:0"],
      leaderCenter: { x: 0.5, y: 0.9 },
    });
    expect(recordingKeyboardShortcutPositions(movedTogether)).toEqual([
      { centerX: 0.6, centerY: 0.7, segmentId: 0, shortcutId: 4 },
      { centerX: 0.6, centerY: 0.7, segmentId: 0, shortcutId: 6 },
    ]);
  });

  it("resizes selected fragments together and resets all transforms only", () => {
    const deleted = deleteRecordingKeyboardShortcutFragments(
      createRecordingTimelineEdit(7),
      ["8:0"],
    );
    const resized = resizeRecordingKeyboardShortcutFragments({
      center: { x: 0.4, y: 0.6 },
      edit: deleted,
      fragmentIds: ["4:0", "6:0"],
      sizePercent: 135,
    });
    expect(recordingKeyboardShortcutPositions(resized)).toEqual([
      {
        centerX: 0.4,
        centerY: 0.6,
        segmentId: 0,
        shortcutId: 4,
        sizePercent: 135,
      },
      {
        centerX: 0.4,
        centerY: 0.6,
        segmentId: 0,
        shortcutId: 6,
        sizePercent: 135,
      },
    ]);
    const clamped = resizeRecordingKeyboardShortcutFragments({
      center: { x: 0.4, y: 0.6 },
      edit: resized,
      fragmentIds: ["4:0"],
      maximumSizePercent: 180,
      minimumSizePercent: 50,
      sizePercent: 300,
    });
    expect(recordingKeyboardShortcutPositions(clamped)[0]?.sizePercent).toBe(
      180,
    );
    expect(resetAllRecordingKeyboardShortcutPositions(resized)).toEqual(
      deleted,
    );
  });
});
