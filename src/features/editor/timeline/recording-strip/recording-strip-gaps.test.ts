// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { recordingTimelineCuts } from "../editing/recording-timeline-cuts";
import {
  createRecordingTimelineEdit,
  recordingTimelineRetainedDuration,
} from "../editing/recording-timeline-edit";
import { isGapSegment } from "../recording-timeline-types";

import {
  slideRecordingTimelineGap,
  trimRecordingTimelineGap,
} from "./recording-strip-gap-edges";
import {
  deleteRecordingTimelineGap,
  insertRecordingTimelineGap,
  MAX_GAP_MS,
  resizeRecordingTimelineGap,
} from "./recording-strip-gaps";

import type { RecordingAudioClip } from "../../../../bindings/RecordingAudioClip";

const DURATION_MS = 10_000;

const song = (startMs: number): RecordingAudioClip => ({
  endMs: startMs + 2_000,
  id: `song-${String(startMs)}`,
  isInstrumental: false,
  isLooped: false,
  kind: "music",
  lane: 0,
  offsetMs: 0,
  sound: "song",
  startMs,
  volumeDb: 0,
});

const playedMs = (edit: ReturnType<typeof createRecordingTimelineEdit>) =>
  recordingTimelineRetainedDuration(edit) * DURATION_MS;

describe("gaps", () => {
  const edit = {
    ...createRecordingTimelineEdit(1),
    audioClips: [song(1_000), song(6_000)],
  };
  const gapped = insertRecordingTimelineGap(edit, {
    atMs: 4_000,
    durationMs: DURATION_MS,
    lengthMs: 3_000,
  });

  it("splits the recording where one goes and plays it past the end", () => {
    expect(gapped.segments.map(isGapSegment)).toEqual([false, true, false]);
    expect(gapped.segments[0].sourceEnd).toBeCloseTo(0.4);
    expect(gapped.segments[2].sourceStart).toBeCloseTo(0.4);
    expect(playedMs(gapped)).toBeCloseTo(13_000);
    // The empty source past the end is no cut to restore.
    expect(recordingTimelineCuts(gapped)).toEqual([]);
  });

  it("moves the songs after it, and keeps those playing through", () => {
    expect(gapped.audioClips?.map(({ startMs }) => startMs)).toEqual([
      1_000, 9_000,
    ]);
  });

  it("grows inside a gap rather than splitting it", () => {
    const grown = insertRecordingTimelineGap(gapped, {
      atMs: 5_000,
      durationMs: DURATION_MS,
      lengthMs: 1_000,
    });
    expect(grown.segments.filter(isGapSegment)).toHaveLength(1);
    expect(playedMs(grown)).toBeCloseTo(14_000);
  });

  it("gives each gap a stretch of the empty source no other meets", () => {
    const twice = insertRecordingTimelineGap(gapped, {
      atMs: 0,
      durationMs: DURATION_MS,
      lengthMs: 2_000,
    });
    const gaps = twice.segments
      .filter(isGapSegment)
      .sort((one, other) => one.sourceStart - other.sourceStart);
    expect(gaps).toHaveLength(2);
    expect(gaps[0].sourceEnd).toBeLessThan(gaps[1].sourceStart);
    expect(twice.segments[0]).toBe(
      gaps.find((gap) => gap.id === twice.segments[0].id),
    );
  });

  it("resizes, moving what comes after with its end", () => {
    const id = gapped.segments[1].id;
    const shorter = resizeRecordingTimelineGap(gapped, {
      durationMs: DURATION_MS,
      id,
      lengthMs: 1_000,
    });
    expect(playedMs(shorter)).toBeCloseTo(11_000);
    expect(shorter.audioClips?.map(({ startMs }) => startMs)).toEqual([
      1_000, 7_000,
    ]);
  });

  it("closes up when deleted, the recording made one again", () => {
    const removed = deleteRecordingTimelineGap(gapped, {
      durationMs: DURATION_MS,
      id: gapped.segments[1].id,
    });
    expect(removed.segments).toEqual([
      expect.objectContaining({ sourceEnd: 1, sourceStart: 0 }),
    ]);
    expect(removed.audioClips?.map(({ startMs }) => startMs)).toEqual([
      1_000, 6_000,
    ]);
  });

  it("is drawn out by its end, what follows moving with it", () => {
    const gap = gapped.segments[1];
    const trim = (from: typeof gapped, atMs: number) =>
      trimRecordingTimelineGap(from, {
        atMs,
        durationMs: DURATION_MS,
        edge: "end",
        id: gap.id,
      });
    const endMs = gap.sourceEnd * DURATION_MS;
    const longer = trim(gapped, endMs + 2_000);
    expect(playedMs(longer)).toBeCloseTo(15_000);
    expect(longer.audioClips?.map((clip) => clip.startMs)).toEqual([
      1_000, 11_000,
    ]);
    // A drag tells where its end is, so telling it twice changes nothing.
    expect(trim(longer, endMs + 2_000)).toEqual(longer);
  });

  it("is drawn out by its start, nothing of the recording cut", () => {
    const gap = gapped.segments[1];
    const trim = (from: typeof gapped, atMs: number) =>
      trimRecordingTimelineGap(from, {
        atMs,
        durationMs: DURATION_MS,
        edge: "start",
        id: gap.id,
      });
    const startMs = gap.sourceStart * DURATION_MS;
    const earlier = trim(gapped, startMs - 1_000);
    expect(playedMs(earlier)).toBeCloseTo(14_000);
    expect(earlier.segments[0].sourceEnd).toBeCloseTo(0.4);
    expect(earlier.audioClips?.map((clip) => clip.startMs)).toEqual([
      1_000, 10_000,
    ]);
    expect(trim(earlier, startMs - 1_000)).toEqual(earlier);
    expect(playedMs(trim(earlier, startMs + 1_000))).toBeCloseTo(12_000);
  });

  it("slides whole, the recording split wherever it goes", () => {
    const id = gapped.segments[1].id;
    const slide = (from: typeof gapped, atMs: number) =>
      slideRecordingTimelineGap(from, { atMs, durationMs: DURATION_MS, id });
    const early = slide(gapped, 2_000);
    expect(early.segments.map(isGapSegment)).toEqual([false, true, false]);
    expect(early.segments[0].sourceEnd).toBeCloseTo(0.2);
    expect(early.segments[1].id).toBe(id);
    expect(early.audioClips?.map(({ startMs }) => startMs)).toEqual([
      1_000, 9_000,
    ]);
    // A drag tells where it starts, so telling it twice changes nothing,
    // and sliding back puts the recording together as it was.
    expect(slide(early, 2_000)).toBe(early);
    const back = slide(early, 4_000);
    expect(back.segments.map(isGapSegment)).toEqual([false, true, false]);
    expect(back.segments[0].sourceEnd).toBeCloseTo(0.4);
    expect(back.audioClips).toEqual(gapped.audioClips);
    const last = slide(gapped, 1e9);
    expect(last.segments.map(isGapSegment)).toEqual([false, true]);
    expect(last.audioClips?.map(({ startMs }) => startMs)).toEqual([
      1_000, 6_000,
    ]);
  });

  it("is drawn out or in at its end, never past the longest gap", () => {
    const gap = gapped.segments[1];
    const endMs = gap.sourceEnd * DURATION_MS;
    const trim = (atMs: number) =>
      trimRecordingTimelineGap(gapped, {
        atMs,
        durationMs: DURATION_MS,
        edge: "end",
        id: gap.id,
      });
    expect(playedMs(trim(endMs - 2_500))).toBeCloseTo(10_500);
    expect(playedMs(trim(endMs + 1e9))).toBeCloseTo(DURATION_MS + MAX_GAP_MS);
  });
});
