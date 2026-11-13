// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { AudioClipKind } from "../../../bindings/AudioClipKind";
import type { RecordingAudioClip } from "../../../bindings/RecordingAudioClip";
import type { Sound } from "../../../bindings/Sound";

// Sounds dragged onto the timeline land where they are let go, one after
// another in the order they came, as the lane they were let go on makes
// them. Sound effects keep their whole length, past the timeline's end
// too, where they are not heard. A song lands in the room nearest where it
// was let go, as a song moved on its rail does: there where it fits, else
// against the songs it would land on, and is cut off at the timeline's end.
// A sound dropped where no lane says what it is goes by its length.

/** A sound this long or longer, dropped where no lane says, is a song. */
const SONG_MS = 10_000;

/** What a sound dropped where no lane says what it is becomes. */
export const guessedKind = ({ durationMs }: Sound): AudioClipKind =>
  durationMs >= SONG_MS ? "music" : "effect";

const clipOf = (
  sound: Sound,
  kind: AudioClipKind,
  { endMs, startMs }: { endMs: number; startMs: number },
): RecordingAudioClip => ({
  endMs,
  id: crypto.randomUUID(),
  isInstrumental: false,
  isLooped: false,
  kind,
  lane: 0,
  offsetMs: 0,
  sound: sound.name,
  startMs,
  volumeDb: 0,
});

/** Where a song `lengthMs` long goes nearest `atMs` among `songs`, cut off
 * at `timelineMs`, or null where no room is left. */
function nearestSongRoom(
  songs: RecordingAudioClip[],
  {
    atMs,
    lengthMs,
    timelineMs,
  }: { atMs: number; lengthMs: number; timelineMs: number },
) {
  const span = (startMs: number) => ({
    endMs: Math.min(startMs + lengthMs, timelineMs),
    startMs,
  });
  const fits = (startMs: number) => {
    const { endMs } = span(startMs);
    return (
      startMs >= 0 &&
      startMs < timelineMs &&
      !songs.some((song) => song.startMs < endMs && song.endMs > startMs)
    );
  };
  const nearest = [
    atMs,
    ...songs.flatMap((song) => [song.startMs - lengthMs, song.endMs]),
  ]
    .map(Math.round)
    .filter(fits)
    .sort((one, other) => Math.abs(one - atMs) - Math.abs(other - atMs))[0] as
    number | undefined;
  return nearest === undefined ? null : span(nearest);
}

/** `clips` with `sounds` let go at `atMs` as `kind`, one after another, and
 * the ones added, in order; a sound with no room, or starting past the
 * timeline's end, is left out. */
export function droppedClips(
  clips: RecordingAudioClip[],
  sounds: Sound[],
  {
    atMs,
    kind,
    timelineMs,
  }: { atMs: number; kind: AudioClipKind; timelineMs: number },
) {
  let all = clips;
  const added: RecordingAudioClip[] = [];
  let next = Math.max(0, Math.round(atMs));
  for (const sound of sounds) {
    const span =
      kind === "music"
        ? nearestSongRoom(
            all.filter((clip) => clip.kind === "music"),
            { atMs: next, lengthMs: sound.durationMs, timelineMs },
          )
        : next < timelineMs
          ? { endMs: next + sound.durationMs, startMs: next }
          : null;
    if (!span) continue;
    const clip = clipOf(sound, kind, span);
    all = [...all, clip];
    added.push(clip);
    next = span.endMs;
  }
  return { added, clips: all };
}
