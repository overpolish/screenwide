// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  recordingTimelineCuts,
  restoreRecordingTimelineCut,
} from "./recording-timeline-cuts";
import {
  createRecordingTimelineEdit,
  RecordingTimelineEdit,
  setRecordingTimelineSegmentPlaybackRate,
} from "./recording-timeline-edit";
import {
  recordingTimelineSilenceSummary,
  removeRecordingTimelineSilences,
  restoreRecordingTimelineSilences,
} from "./recording-timeline-silences";

/** A 100-second recording, so a millisecond is a hundred-thousandth. */
const DURATION_MS = 100_000;

const removed = (edit: RecordingTimelineEdit) =>
  removeRecordingTimelineSilences(
    edit,
    [
      { endMs: 20_000, startMs: 10_000 },
      { endMs: 60_000, startMs: 50_000 },
    ],
    DURATION_MS,
  );

describe("removeRecordingTimelineSilences", () => {
  it("cuts each pause out and records what it took", () => {
    const edit = removed(createRecordingTimelineEdit(1));
    expect(
      edit.segments.map(({ sourceEnd, sourceStart }) => [
        sourceStart,
        sourceEnd,
      ]),
    ).toEqual([
      [0, 0.1],
      [0.2, 0.5],
      [0.6, 1],
    ]);
    expect(recordingTimelineSilenceSummary(edit, DURATION_MS)).toEqual({
      count: 2,
      durationMs: 20_000,
    });
  });

  it("only takes what the edit still keeps", () => {
    // 15 to 30 s is already gone, so only 10 to 15 s of the first pause is
    // left to cut.
    const trimmed: RecordingTimelineEdit = {
      ...createRecordingTimelineEdit(1),
      nextSegmentId: 2,
      segments: [
        { id: 0, sourceEnd: 0.15, sourceStart: 0 },
        { id: 1, sourceEnd: 1, sourceStart: 0.3 },
      ],
    };
    const edit = removed(trimmed);
    expect(edit.silenceCuts?.map((cut) => cut.sourceStart)).toEqual([0.1, 0.5]);
    expect(
      recordingTimelineSilenceSummary(edit, DURATION_MS).durationMs,
    ).toBeCloseTo(15_000);
  });

  it("measures a pause cut from a sped-up clip at its rate", () => {
    const sped = setRecordingTimelineSegmentPlaybackRate(
      createRecordingTimelineEdit(1),
      0,
      2,
    );
    const edit = removed(sped);
    expect(
      recordingTimelineSilenceSummary(edit, DURATION_MS).durationMs,
    ).toBeCloseTo(10_000);
  });
});

describe("restoreRecordingTimelineSilences", () => {
  it("puts the recording back as it was", () => {
    const restored = restoreRecordingTimelineSilences(
      removed(createRecordingTimelineEdit(1)),
    );
    expect(restored.segments).toEqual([
      { id: 0, sourceEnd: 1, sourceStart: 0 },
    ]);
    expect(restored.silenceCuts).toBeUndefined();
  });

  it("brings back each pause at the rate it played at", () => {
    const sped = setRecordingTimelineSegmentPlaybackRate(
      createRecordingTimelineEdit(1),
      0,
      2,
    );
    const restored = restoreRecordingTimelineSilences(removed(sped));
    expect(restored.segments).toEqual([
      { id: 0, playbackRate: 2, sourceEnd: 1, sourceStart: 0 },
    ]);
  });

  it("stops counting a pause restored at its join", () => {
    const edit = removed(createRecordingTimelineEdit(1));
    const one = restoreRecordingTimelineCut(
      edit,
      recordingTimelineCuts(edit)[0],
    );
    const summary = recordingTimelineSilenceSummary(one, DURATION_MS);
    expect(summary.count).toBe(1);
    expect(summary.durationMs).toBeCloseTo(10_000);
    expect(
      restoreRecordingTimelineSilences(one).segments.map(
        ({ sourceEnd, sourceStart }) => [sourceStart, sourceEnd],
      ),
    ).toEqual([[0, 1]]);
  });

  it("leaves a cut of your own alone", () => {
    const own: RecordingTimelineEdit = {
      ...createRecordingTimelineEdit(1),
      nextSegmentId: 2,
      segments: [
        { id: 0, sourceEnd: 0.8, sourceStart: 0 },
        { id: 1, sourceEnd: 1, sourceStart: 0.9 },
      ],
    };
    const restored = restoreRecordingTimelineSilences(removed(own));
    expect(
      restored.segments.map(({ sourceEnd, sourceStart }) => [
        sourceStart,
        sourceEnd,
      ]),
    ).toEqual([
      [0, 0.8],
      [0.9, 1],
    ]);
  });
});
