// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { droppedClips } from "./dropped-clips";

import type { RecordingAudioClip } from "../../../bindings/RecordingAudioClip";
import type { Sound } from "../../../bindings/Sound";

const sound = (durationMs: number): Sound => ({
  addedMs: 0,
  durationMs,
  levels: [],
  loudestLufs: -11,
  loudnessLufs: -14,
  name: "0123456789abcdef0123456789abcdef.mp3",
  title: "Sound",
});

const song = (startMs: number, endMs: number): RecordingAudioClip => ({
  endMs,
  id: `song-${String(startMs)}`,
  isInstrumental: false,
  isLooped: false,
  kind: "music",
  lane: 0,
  offsetMs: 0,
  sound: "0123456789abcdef0123456789abcdef.mp3",
  startMs,
  volumeDb: 0,
});

const spans = (clips: RecordingAudioClip[]) =>
  clips.map(({ endMs, startMs }) => [startMs, endMs]);

describe("sounds dropped on the timeline", () => {
  it("lay sound effects one after another from where they are let go", () => {
    const { added } = droppedClips([], [sound(2_000), sound(11_000)], {
      atMs: 75_000,
      kind: "effect",
      timelineMs: 80_000,
    });
    // The second starts inside the timeline, so it keeps its whole length.
    expect(spans(added)).toEqual([
      [75_000, 77_000],
      [77_000, 88_000],
    ]);
  });

  it("put a song in the room nearest where it is let go", () => {
    const songs = [song(0, 30_000), song(40_000, 70_000)];
    // Let go over the first song's end: the room after it, which is nearer
    // than the room before the start of the timeline.
    const { added } = droppedClips(songs, [sound(5_000)], {
      atMs: 28_000,
      kind: "music",
      timelineMs: 100_000,
    });
    expect(spans(added)).toEqual([[30_000, 35_000]]);
  });

  it("cut a song off at the timeline's end, and leave out one with no room", () => {
    const { added } = droppedClips(
      [song(0, 90_000)],
      [sound(20_000), sound(20_000)],
      { atMs: 95_000, kind: "music", timelineMs: 100_000 },
    );
    expect(spans(added)).toEqual([[95_000, 100_000]]);
  });
});
