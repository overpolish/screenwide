// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { AudioClipKind } from "../../../bindings/AudioClipKind";
import type { RecordingAudioClip } from "../../../bindings/RecordingAudioClip";
import type { Sound } from "../../../bindings/Sound";

// The rules audio clips are laid on the timeline by, in milliseconds of the
// edited video. Clips stack in lanes, as annotations do. Sound effects go
// anywhere; songs never overlap, and those that touch crossfade. A song
// added goes where it is put, unless that lands it on another song: then
// straight after that one, or after the last song where there is no room
// there. Moved or trimmed, a clip stops at the timeline's ends, and a song
// at its neighbours, rather than being cut or pushed over them. One left
// past the end by a shorter timeline is cut off where the timeline ends,
// and played whole again if it grows back.

export type { AudioClipKind, RecordingAudioClip };

/** A sound this long or longer is a song; a shorter one a sound effect, as
 * `screenwide_sound_track::clip::SONG_MS` has it. */
export const SONG_MS = 10_000;
/** The shortest a clip can be trimmed to. */
export const LEAST_CLIP_MS = 100;

export const kindOf = (sound: Pick<Sound, "durationMs">): AudioClipKind =>
  sound.durationMs >= SONG_MS ? "music" : "effect";

type Stretch = Pick<RecordingAudioClip, "endMs" | "startMs">;

const overlaps = (clip: Stretch, startMs: number, endMs: number) =>
  clip.startMs < endMs && clip.endMs > startMs;

/** Where a song `lengthMs` long lands when put at `atMs`: there where it
 * fits, else straight after the song it lands on, else after the last
 * song. */
export function songStart(
  clips: RecordingAudioClip[],
  { atMs, lengthMs }: { atMs: number; lengthMs: number },
) {
  const songs = clips
    .filter((clip) => clip.kind === "music")
    .sort((one, other) => one.startMs - other.startMs);
  const fits = (startMs: number) =>
    !songs.some((song) => overlaps(song, startMs, startMs + lengthMs));
  const start = Math.max(0, Math.round(atMs));
  if (fits(start)) return start;
  const landedOn = songs.find((song) =>
    overlaps(song, start, start + lengthMs),
  );
  if (landedOn && fits(landedOn.endMs)) return landedOn.endMs;
  return Math.max(...songs.map((song) => song.endMs));
}

/** A clip of all of `sound`, laid at `atMs` by its kind's rules, never past
 * `timelineMs`; null where it would start at or past it. It goes in the
 * top lane, drawn in front, and the stack settles round it. */
export function placedClip(
  clips: RecordingAudioClip[],
  sound: Sound,
  { atMs, timelineMs }: { atMs: number; timelineMs: number },
): RecordingAudioClip | null {
  const kind = kindOf(sound);
  const lengthMs = sound.durationMs;
  const startMs =
    kind === "music"
      ? songStart(clips, { atMs, lengthMs })
      : Math.max(0, Math.round(atMs));
  if (startMs >= timelineMs) return null;
  return {
    endMs: Math.min(startMs + lengthMs, timelineMs),
    id: crypto.randomUUID(),
    isInstrumental: false,
    isLooped: false,
    kind,
    lane: 0,
    offsetMs: 0,
    sound: sound.name,
    startMs,
    volumeDb: 0,
  };
}

/** The room a song has between the songs either side of it, which it may
 * touch but not cross; a sound effect's is the whole timeline. */
export function songRoom(
  clips: RecordingAudioClip[],
  clip: RecordingAudioClip,
) {
  const others =
    clip.kind === "music"
      ? clips.filter((other) => other.kind === "music" && other.id !== clip.id)
      : [];
  return {
    from: Math.max(
      0,
      ...others
        .filter((other) => other.startMs < clip.startMs)
        .map((other) => other.endMs),
    ),
    to: Math.min(
      Number.POSITIVE_INFINITY,
      ...others
        .filter((other) => other.startMs >= clip.startMs)
        .map((other) => other.startMs),
    ),
  };
}

/** `clip` moved by `deltaMs`, as long as it was, kept inside the timeline.
 * A song never lands on another: where it would, it sits in the nearest
 * room beside the songs it lands on, so pushed into a neighbour it stops
 * against it, and pushed past its middle it jumps to its far side, which
 * is how songs are put in another order. */
export function movedClip(
  clips: RecordingAudioClip[],
  clip: RecordingAudioClip,
  { deltaMs, timelineMs }: { deltaMs: number; timelineMs: number },
): RecordingAudioClip {
  const lengthMs = clip.endMs - clip.startMs;
  const latest = timelineMs - lengthMs;
  if (latest < 0) return clip;
  const at = (startMs: number) => ({
    ...clip,
    endMs: startMs + lengthMs,
    startMs,
  });
  const wanted = Math.round(
    Math.min(Math.max(clip.startMs + deltaMs, 0), latest),
  );
  if (clip.kind !== "music") return at(wanted);
  const songs = clips.filter(
    (other) => other.kind === "music" && other.id !== clip.id,
  );
  const fits = (startMs: number) =>
    startMs >= 0 &&
    startMs <= latest &&
    !songs.some((song) => overlaps(song, startMs, startMs + lengthMs));
  if (fits(wanted)) return at(wanted);
  const nearest = songs
    .flatMap((song) => [song.startMs - lengthMs, song.endMs])
    .filter(fits)
    .sort(
      (one, other) => Math.abs(one - wanted) - Math.abs(other - wanted),
    )[0] as number | undefined;
  return nearest === undefined ? clip : at(nearest);
}

/** How far `clip`'s `edge` can go: never before its sound's start or past
 * its end unless it loops, never past the timeline or, for a song, into
 * the songs either side, and never shorter than the least clip. */
export function trimBounds(
  clips: RecordingAudioClip[],
  clip: RecordingAudioClip,
  {
    durationMs,
    edge,
    timelineMs,
  }: { durationMs: number; edge: "end" | "start"; timelineMs: number },
) {
  const room = songRoom(clips, clip);
  if (edge === "start")
    return {
      max: clip.endMs - LEAST_CLIP_MS,
      min: Math.max(clip.startMs - clip.offsetMs, room.from),
    };
  const soundEnd = clip.isLooped
    ? Number.POSITIVE_INFINITY
    : clip.startMs + durationMs - clip.offsetMs;
  return {
    max: Math.min(soundEnd, timelineMs, room.to),
    min: clip.startMs + LEAST_CLIP_MS,
  };
}

/** `clip` with `edge` trimmed to `atMs` within its bounds: the start takes
 * the sound's offset with it, so what plays stays where it was. */
export function trimmedClip(
  clip: RecordingAudioClip,
  {
    atMs,
    bounds,
    edge,
  }: {
    atMs: number;
    bounds: { max: number; min: number };
    edge: "end" | "start";
  },
): RecordingAudioClip {
  const to = Math.round(Math.min(Math.max(atMs, bounds.min), bounds.max));
  if (edge === "end") return { ...clip, endMs: to };
  return {
    ...clip,
    offsetMs: Math.max(0, clip.offsetMs + to - clip.startMs),
    startMs: to,
  };
}

/** `clip` playing its sound from `offsetMs` in, as long as it was: one
 * that does not loop starts no later than leaves it room to play to its
 * end. */
export function startedFrom(
  clip: RecordingAudioClip,
  offsetMs: number,
  durationMs: number,
): RecordingAudioClip {
  const latest = clip.isLooped
    ? durationMs - LEAST_CLIP_MS
    : durationMs - (clip.endMs - clip.startMs);
  return {
    ...clip,
    offsetMs: Math.round(Math.min(Math.max(offsetMs, 0), Math.max(0, latest))),
  };
}

/** `clip` looping or not: one that stops looping ends where its sound
 * does, if that is sooner. */
export function loopedClip(
  clip: RecordingAudioClip,
  isLooped: boolean,
  durationMs: number,
): RecordingAudioClip {
  if (isLooped) return { ...clip, isLooped };
  return {
    ...clip,
    endMs: Math.min(clip.endMs, clip.startMs + durationMs - clip.offsetMs),
    isLooped,
  };
}
