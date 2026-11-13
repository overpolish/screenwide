// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  keptStretch,
  loopedClip,
  movedClip,
  placedClip,
  type RecordingAudioClip,
  retypedClip,
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
  loudestLufs: -11,
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

  it("stops a song at the timeline's ends rather than cutting it", () => {
    const song = clip("s", [5_000, 8_000]);
    const back = movedClip([], song, { deltaMs: -9_000, timelineMs: 100_000 });
    const on = movedClip([], song, { deltaMs: 99_000, timelineMs: 100_000 });
    expect([back.startMs, back.endMs, on.startMs, on.endMs]).toEqual([
      0, 3_000, 97_000, 100_000,
    ]);
  });

  it("moves a sound effect past the end, keeping its length", () => {
    // Eleven seconds laid 4 s from the end runs past it, and can still go
    // later, until only the least of it is left inside.
    const effect = clip("hit", [76_000, 87_000], { kind: "effect" });
    const later = movedClip([], effect, { deltaMs: 2_000, timelineMs: 80_000 });
    const last = movedClip([], effect, { deltaMs: 9_000, timelineMs: 80_000 });
    expect([later.startMs, later.endMs]).toEqual([78_000, 89_000]);
    expect([last.startMs, last.endMs]).toEqual([79_900, 90_900]);
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
  it("never runs a song past the timeline", () => {
    const placed = placedClip([], sound(60_000), {
      atMs: 50_000,
      kind: "music",
      timelineMs: 80_000,
    });
    expect([placed?.kind, placed?.startMs, placed?.endMs]).toEqual([
      "music",
      50_000,
      80_000,
    ]);
    expect(
      placedClip([], sound(1_000), {
        atMs: 80_000,
        kind: "effect",
        timelineMs: 80_000,
      }),
    ).toBeNull();
  });

  it("is the kind it is placed as, whatever its length", () => {
    const songs = [clip("a", [0, 60_000])];
    // Eleven seconds as a sound effect lands where it is put, over a song;
    // as a song it goes after the song it lands on.
    const effect = placedClip(songs, sound(11_000), {
      atMs: 10_000,
      kind: "effect",
      timelineMs: 80_000,
    });
    expect([effect?.kind, effect?.startMs]).toEqual(["effect", 10_000]);
    const song = placedClip(songs, sound(11_000), {
      atMs: 10_000,
      kind: "music",
      timelineMs: 80_000,
    });
    expect([song?.kind, song?.startMs]).toEqual(["music", 60_000]);
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

describe("sound effects", () => {
  const effect = clip("e", [10_000, 12_000], { kind: "effect" });

  it("plays the stretch its island keeps, inside its sound", () => {
    const kept = keptStretch(
      effect,
      { lengthMs: 1_500, offsetMs: 300 },
      { durationMs: 3_000, timelineMs: 60_000 },
    );
    expect([kept.startMs, kept.endMs, kept.offsetMs]).toEqual([
      10_000, 11_500, 300,
    ]);
    const past = keptStretch(
      effect,
      { lengthMs: 9_000, offsetMs: 2_000 },
      { durationMs: 3_000, timelineMs: 60_000 },
    );
    expect([past.endMs, past.offsetMs]).toEqual([11_000, 2_000]);
  });

  it("becomes a song where it lands clear of the songs, or after the one it lands on", () => {
    const songs = [clip("a", [0, 30_000]), clip("b", [40_000, 70_000])];
    expect(
      retypedClip(songs, clip("e", [32_000, 34_000], { kind: "effect" }), {
        kind: "music",
        timelineMs: 90_000,
      }).startMs,
    ).toBe(32_000);
    const moved = retypedClip(songs, effect, {
      kind: "music",
      timelineMs: 90_000,
    });
    expect([moved.kind, moved.startMs, moved.endMs]).toEqual([
      "music",
      30_000,
      32_000,
    ]);
  });

  it("stops looping when it becomes a sound effect", () => {
    const song = clip("s", [0, 30_000], {
      isInstrumental: true,
      isLooped: true,
    });
    expect(
      retypedClip([], song, { kind: "effect", timelineMs: 90_000 }),
    ).toMatchObject({ isInstrumental: false, isLooped: false, kind: "effect" });
  });
});
