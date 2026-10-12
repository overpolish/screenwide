// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  createRecordingTimelineEdit,
  RecordingTimelineEdit,
} from "../../timeline/editing/recording-timeline-edit";
import {
  removeRecordingTimelineSilences,
  restoreRecordingTimelineSilences,
} from "../../timeline/editing/recording-timeline-silences";

import {
  applyRecordingTranscriptCorrection,
  applyRecordingTranscriptCorrections,
  cutRecordingTranscriptRanges,
  recordingTranscriptCanRestore,
  recordingTranscriptCorrectionsMs,
  recordingTranscriptRemovedMs,
  restoreAllRecordingTranscript,
  restoreRecordingTranscriptRanges,
} from "./recording-timeline-transcript";

import type { TranscriptWord } from "../../tool-panels/transcript/transcript-words";

/** A 100-second recording, so a millisecond is a hundred-thousandth. */
const DURATION_MS = 100_000;

const words: TranscriptWord[] = [
  { endMs: 5_000, probability: 0.9, startMs: 4_000, text: "hello" },
  { endMs: 7_000, probability: 0.9, startMs: 6_000, text: "world" },
];

/** Correct `edit` with `change`, in the recording these words came from. */
const correct = (
  edit: RecordingTimelineEdit,
  change: { endMs: number; startMs: number; text: string },
) =>
  applyRecordingTranscriptCorrection(edit, change, {
    sourceDurationMs: DURATION_MS,
    words,
  });

const segments = (edit: RecordingTimelineEdit) =>
  edit.segments.map(({ sourceEnd, sourceStart }) => [sourceStart, sourceEnd]);

describe("cutRecordingTranscriptRanges", () => {
  it("cuts each stretch out and records what it took", () => {
    const edit = cutRecordingTranscriptRanges(
      createRecordingTimelineEdit(1),
      [
        { endMs: 20_000, startMs: 10_000 },
        { endMs: 60_000, startMs: 50_000 },
      ],
      DURATION_MS,
    );
    expect(segments(edit)).toEqual([
      [0, 0.1],
      [0.2, 0.5],
      [0.6, 1],
    ]);
    expect(edit.transcriptCuts?.map((cut) => cut.sourceStart)).toEqual([
      0.1, 0.5,
    ]);
    expect(recordingTranscriptCanRestore(edit)).toBe(true);
  });

  it("only takes what the edit still keeps", () => {
    const trimmed: RecordingTimelineEdit = {
      ...createRecordingTimelineEdit(1),
      nextSegmentId: 2,
      segments: [
        { id: 0, sourceEnd: 0.15, sourceStart: 0 },
        { id: 1, sourceEnd: 1, sourceStart: 0.3 },
      ],
    };
    const edit = cutRecordingTranscriptRanges(
      trimmed,
      [{ endMs: 20_000, startMs: 10_000 }],
      DURATION_MS,
    );
    expect(edit.transcriptCuts?.map((cut) => cut.sourceStart)).toEqual([0.1]);
  });
});

describe("restoreRecordingTranscriptRanges", () => {
  it("brings back a cut of the panel's and clears its record", () => {
    const cut = cutRecordingTranscriptRanges(
      createRecordingTimelineEdit(1),
      [{ endMs: 20_000, startMs: 10_000 }],
      DURATION_MS,
    );
    const restored = restoreRecordingTranscriptRanges(
      cut,
      [{ endMs: 20_000, startMs: 10_000 }],
      DURATION_MS,
    );
    expect(segments(restored)).toEqual([[0, 1]]);
    expect(restored.transcriptCuts).toBeUndefined();
  });

  it("brings back a cut of your own at the rate beside it", () => {
    const own: RecordingTimelineEdit = {
      ...createRecordingTimelineEdit(1),
      nextSegmentId: 2,
      segments: [
        { id: 0, sourceEnd: 0.8, sourceStart: 0 },
        { id: 1, sourceEnd: 1, sourceStart: 0.9 },
      ],
    };
    const restored = restoreRecordingTranscriptRanges(
      own,
      [{ endMs: 90_000, startMs: 80_000 }],
      DURATION_MS,
    );
    expect(segments(restored)).toEqual([[0, 1]]);
  });

  it("brings back a silence Remove cut and trims its record", () => {
    const silenced = removeRecordingTimelineSilences(
      createRecordingTimelineEdit(1),
      [{ endMs: 20_000, startMs: 10_000 }],
      DURATION_MS,
    );
    const restored = restoreRecordingTranscriptRanges(
      silenced,
      [{ endMs: 20_000, startMs: 10_000 }],
      DURATION_MS,
    );
    expect(segments(restored)).toEqual([[0, 1]]);
    expect(restored.silenceCuts).toBeUndefined();
  });

  it("leaves a cut outside the ranges alone", () => {
    const edit = cutRecordingTranscriptRanges(
      createRecordingTimelineEdit(1),
      [
        { endMs: 20_000, startMs: 10_000 },
        { endMs: 60_000, startMs: 50_000 },
      ],
      DURATION_MS,
    );
    const restored = restoreRecordingTranscriptRanges(
      edit,
      [{ endMs: 20_000, startMs: 10_000 }],
      DURATION_MS,
    );
    expect(segments(restored)).toEqual([
      [0, 0.5],
      [0.6, 1],
    ]);
    expect(restored.transcriptCuts?.map((cut) => cut.sourceStart)).toEqual([
      0.5,
    ]);
  });
});

describe("restoreAllRecordingTranscript", () => {
  it("brings back only what the panel cut, leaving silence cuts out", () => {
    const edit = removeRecordingTimelineSilences(
      cutRecordingTranscriptRanges(
        createRecordingTimelineEdit(1),
        [{ endMs: 30_000, startMs: 20_000 }],
        DURATION_MS,
      ),
      [{ endMs: 60_000, startMs: 50_000 }],
      DURATION_MS,
    );
    const restored = restoreAllRecordingTranscript(edit);
    expect(segments(restored)).toEqual([
      [0, 0.5],
      [0.6, 1],
    ]);
    expect(restored.transcriptCuts).toBeUndefined();
    expect(restored.silenceCuts?.map((cut) => cut.sourceStart)).toEqual([0.5]);
  });
});

describe("recordingTranscriptRemovedMs", () => {
  it("reports every cut as a source stretch in milliseconds", () => {
    const edit = cutRecordingTranscriptRanges(
      createRecordingTimelineEdit(1),
      [{ endMs: 20_000, startMs: 10_000 }],
      DURATION_MS,
    );
    expect(recordingTranscriptRemovedMs(edit, DURATION_MS)).toEqual([
      { endMs: 20_000, startMs: 10_000 },
    ]);
  });
});

describe("applyRecordingTranscriptCorrection", () => {
  it("records a correction in shares and reads it back in rounded ms", () => {
    const edit = correct(createRecordingTimelineEdit(1), {
      endMs: 7_000,
      startMs: 4_000,
      text: "Greetings",
    });
    expect(edit.transcriptCorrections).toEqual([
      { sourceEnd: 0.07, sourceStart: 0.04, text: "Greetings" },
    ]);
    expect(recordingTranscriptCorrectionsMs(edit, DURATION_MS)).toEqual([
      { endMs: 7_000, startMs: 4_000, text: "Greetings" },
    ]);
  });

  it("takes a correction away when its text is what was heard", () => {
    const corrected = correct(createRecordingTimelineEdit(1), {
      endMs: 7_000,
      startMs: 4_000,
      text: "Greetings",
    });
    const restored = correct(corrected, {
      endMs: 7_000,
      startMs: 4_000,
      text: "hello world",
    });
    expect(restored.transcriptCorrections).toEqual([]);
  });

  it("replaces every correction the new one overlaps", () => {
    const first = correct(createRecordingTimelineEdit(1), {
      endMs: 5_000,
      startMs: 4_000,
      text: "Hi",
    });
    const second = correct(first, {
      endMs: 7_000,
      startMs: 4_000,
      text: "Greetings",
    });
    expect(second.transcriptCorrections).toEqual([
      { sourceEnd: 0.07, sourceStart: 0.04, text: "Greetings" },
    ]);
  });
});

describe("applyRecordingTranscriptCorrections", () => {
  it("applies every change in one call, each in turn", () => {
    const edit = applyRecordingTranscriptCorrections(
      createRecordingTimelineEdit(1),
      [
        { endMs: 5_000, startMs: 4_000, text: "Hi" },
        { endMs: 7_000, startMs: 6_000, text: "Earth" },
      ],
      { sourceDurationMs: DURATION_MS, words },
    );
    expect(recordingTranscriptCorrectionsMs(edit, DURATION_MS)).toEqual([
      { endMs: 5_000, startMs: 4_000, text: "Hi" },
      { endMs: 7_000, startMs: 6_000, text: "Earth" },
    ]);
  });

  it("leaves the edit unchanged for no changes", () => {
    const edit = createRecordingTimelineEdit(1);
    expect(
      applyRecordingTranscriptCorrections(edit, [], {
        sourceDurationMs: DURATION_MS,
        words,
      }),
    ).toBe(edit);
  });
});

describe("restoreRecordingTimelineSilences regression", () => {
  it("still puts the recording back as it was", () => {
    const removed = removeRecordingTimelineSilences(
      createRecordingTimelineEdit(1),
      [
        { endMs: 20_000, startMs: 10_000 },
        { endMs: 60_000, startMs: 50_000 },
      ],
      DURATION_MS,
    );
    expect(segments(restoreRecordingTimelineSilences(removed))).toEqual([
      [0, 1],
    ]);
  });
});
