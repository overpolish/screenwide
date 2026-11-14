// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { MusicFilters } from "./music-filters";
import type { MusicMood } from "../../../../bindings/MusicMood";
import type { MusicTempo } from "../../../../bindings/MusicTempo";
import type { MusicVocals } from "../../../../bindings/MusicVocals";
import type { Sound } from "../../sound-track/sound-library";

// A song in the library as the music filters see it. One found on Openverse
// is described as Jamendo tagged it, so it stays under the filters it was
// found with. One added from a file has its tempo from the beat measured as
// it was added and its singing as the separation model heard it, once it
// has; it has no mood or genre. What a song lacks, it is left out for while
// that filter is set.

export type SongDescription = {
  /** As Openverse names them, such as `ambient`. */
  genres: readonly string[];
  moods: readonly MusicMood[];
  tempo: MusicTempo | null;
  vocals: MusicVocals | null;
};

/** The fastest tempo that is slow and that is medium, in BPM, roughly
 * where Jamendo's slow and fast tags fall. */
const SLOWEST_MEDIUM_BPM = 90;
const FASTEST_MEDIUM_BPM = 120;

/** The tempo of a song beating once every `periodMs`. */
const tempoOfBeat = (periodMs: number): MusicTempo => {
  const bpm = 60_000 / periodMs;
  if (bpm < SLOWEST_MEDIUM_BPM) return "slow";
  return bpm <= FASTEST_MEDIUM_BPM ? "medium" : "fast";
};

export const describeSong = ({ music, tags }: Sound): SongDescription => {
  const beat = music?.beat;
  return {
    genres: tags?.genres ?? [],
    moods: tags?.moods ?? [],
    tempo: tags?.tempo ?? (beat ? tempoOfBeat(beat.periodMs) : null),
    vocals: tags?.vocals ?? music?.vocals ?? null,
  };
};

/** Whether a song described as `song` is one `filters` let through. */
export const fitsMusicFilters = (
  song: SongDescription,
  { mood, tempo, vocals }: MusicFilters,
) =>
  (tempo === undefined || song.tempo === tempo) &&
  (vocals === undefined || song.vocals === vocals) &&
  (mood === undefined || song.moods.includes(mood));
