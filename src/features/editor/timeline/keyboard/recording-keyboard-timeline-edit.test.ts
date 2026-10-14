// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { createRecordingTimelineEdit } from "../editing/recording-timeline-edit";

import { cutKeyboardTimeline } from "./recording-keyboard-timeline-cut";
import {
  deletedRecordingKeyboardShortcutRanges,
  deleteRecordingKeyboardShortcutFragments,
  restoreRecordingKeyboardShortcuts,
} from "./recording-keyboard-timeline-edit";

describe("recording keyboard shortcut deletion", () => {
  it("copies fragment deletions into a bladed segment", () => {
    const edited = {
      ...createRecordingTimelineEdit(7),
      deletedKeyboardShortcutFragments: [{ segmentId: 0, shortcutId: 8 }],
    };
    const cut = cutKeyboardTimeline(edited, 0.5);

    expect(cut).toMatchObject({
      deletedKeyboardShortcutFragments: [
        { segmentId: 0, shortcutId: 8 },
        { segmentId: 1, shortcutId: 8 },
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
