// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  loopedClip,
  movedClip,
  placedClip,
  type RecordingAudioClip,
  songStart,
  startedFrom,
  trimBounds,
  trimmedClip,
} from "./audio-clips";

import type { Sound } from "../../../bindings/Sound";

const clip = (
  id: string,
  [startMs, endMs]: [number, number],
  extra: Partial<RecordingAudioClip> = {},
): RecordingAudioClip => ({
  endMs,
  id,
  isInstrumental: false,
  isLooped: false,
  kind: "music",
  lane: 0,
  offsetMs: 0,
  sound: "0123456789abcdef0123456789abcdef.mp3",
  startMs,
  volumeDb: 0,
  ...extra,
});

const sound = (durationMs: number): Sound => ({
  addedMs: 0,
  durationMs,
  levels: [],
  loudnessLufs: -14,
  name: "0123456789abcdef0123456789abcdef.mp3",
  title: "Song",
});

describe("songs", () => {
  const songs = [clip("a", [0, 60_000]), clip("b", [90_000, 120_000])];

  it("land where they are dropped where there is room", () => {
    expect(songStart(songs, { atMs: 60_000, lengthMs: 30_000 })).toBe(60_000);
  });

  it("dropped onto a song go straight after it", () => {
    expect(songStart(songs, { atMs: 20_000, lengthMs: 20_000 })).toBe(60_000);
  });

  it("go after the last song when there is no room after the one they land on", () => {
    expect(songStart(songs, { atMs: 20_000, lengthMs: 40_000 })).toBe(120_000);
  });
});

describe("moving a clip on the strip", () => {
  const songs = [
    clip("a", [0, 20_000]),
    clip("b", [30_000, 50_000]),
    clip("c", [70_000, 80_000]),
  ];

  it("keeps its length and its place in its sound", () => {
    const moved = movedClip(songs, songs[1], {
      deltaMs: 5_000,
      timelineMs: 100_000,
    });
    expect([moved.startMs, moved.endMs]).toEqual([35_000, 55_000]);
  });

  it("stops a song against a neighbour it is pushed into", () => {
    const back = movedClip(songs, songs[1], {
      deltaMs: -25_000,
      timelineMs: 100_000,
    });
    const on = movedClip(songs, songs[1], {
      deltaMs: 25_000,
      timelineMs: 100_000,
    });
    expect([back.startMs, on.endMs]).toEqual([20_000, 70_000]);
  });

  it("puts a song pushed past a neighbour's middle on its far side", () => {
    const moved = movedClip(songs, songs[0], {
      deltaMs: 35_000,
      timelineMs: 100_000,
    });
    expect([moved.startMs, moved.endMs]).toEqual([50_000, 70_000]);
  });

  it("stops at the timeline's ends rather than cutting the clip", () => {
    const effect = clip("hit", [5_000, 8_000], { kind: "effect" });
    const back = movedClip(songs, effect, {
      deltaMs: -9_000,
      timelineMs: 100_000,
    });
    const on = movedClip(songs, effect, {
      deltaMs: 99_000,
      timelineMs: 100_000,
    });
    expect([back.startMs, back.endMs, on.startMs, on.endMs]).toEqual([
      0, 3_000, 97_000, 100_000,
    ]);
  });

  it("lets a sound effect pass over songs", () => {
    const effect = clip("hit", [5_000, 8_000], { kind: "effect" });
    expect(
      movedClip(songs, effect, { deltaMs: 30_000, timelineMs: 100_000 })
        .startMs,
    ).toBe(35_000);
  });
});

describe("choosing where in its sound a clip plays from", () => {
  it("keeps the clip as long as it was, inside its sound", () => {
    const later = startedFrom(clip("a", [0, 50_000]), 20_000, 60_000);
    expect([later.offsetMs, later.endMs]).toEqual([10_000, 50_000]);
  });

  it("lets a looped clip start anywhere in its sound", () => {
    const looped = clip("a", [0, 50_000], { isLooped: true });
    expect(startedFrom(looped, 20_000, 30_000).offsetMs).toBe(20_000);
  });
});

describe("a clip placed from the library", () => {
  it("never runs past the timeline", () => {
    const placed = placedClip([], sound(60_000), {
      atMs: 50_000,
      timelineMs: 80_000,
    });
    expect([placed?.kind, placed?.startMs, placed?.endMs]).toEqual([
      "music",
      50_000,
      80_000,
    ]);
    expect(
      placedClip([], sound(1_000), { atMs: 80_000, timelineMs: 80_000 }),
    ).toBeNull();
  });

  it("is a sound effect under ten seconds", () => {
    const placed = placedClip([], sound(4_000), {
      atMs: 0,
      timelineMs: 80_000,
    });
    expect(placed?.kind).toBe("effect");
  });
});

describe("trimming", () => {
  const song = clip("b", [40_000, 60_000], { offsetMs: 5_000 });

  it("reaches back no further than the sound's start or the song before", () => {
    expect(
      trimBounds([song], song, {
        durationMs: 60_000,
        edge: "start",
        timelineMs: 90_000,
      }).min,
    ).toBe(35_000);
    const before = clip("a", [0, 38_000]);
    expect(
      trimBounds([before, song], song, {
        durationMs: 60_000,
        edge: "start",
        timelineMs: 90_000,
      }).min,
    ).toBe(38_000);
  });

  it("reaches on no further than the sound's end unless it loops", () => {
    const bounds = trimBounds([song], song, {
      durationMs: 60_000,
      edge: "end",
      timelineMs: 90_000,
    });
    expect(bounds.max).toBe(90_000);
    const short = clip("c", [0, 10_000]);
    expect(
      trimBounds([short], short, {
        durationMs: 12_000,
        edge: "end",
        timelineMs: 90_000,
      }).max,
    ).toBe(12_000);
    const looped = { ...short, isLooped: true };
    expect(
      trimBounds([looped], looped, {
        durationMs: 12_000,
        edge: "end",
        timelineMs: 90_000,
      }).max,
    ).toBe(90_000);
  });

  it("moves the start with the sound under it", () => {
    const trimmed = trimmedClip(song, {
      atMs: 38_000,
      bounds: { max: 59_900, min: 35_000 },
      edge: "start",
    });
    expect([trimmed.startMs, trimmed.offsetMs]).toEqual([38_000, 3_000]);
  });

  it("stops a clip that no longer loops where its sound ends", () => {
    const looped = clip("a", [0, 50_000], { isLooped: true, offsetMs: 2_000 });
    expect(loopedClip(looped, false, 12_000).endMs).toBe(10_000);
  });
});
