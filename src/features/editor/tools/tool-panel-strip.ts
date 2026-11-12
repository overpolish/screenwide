// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { AudioClipKind } from "../../../bindings/AudioClipKind";
import type { BeatWhat } from "../island/beat/beat-island";
import type { LullPlay } from "../island/lull/lull-island";
import type { NoteTranscript } from "../timeline/moments/recording-moments";

/** What is chosen on the strip, where it has an island: one beat, lull,
 * moment, drawn range or song or sound effect, named by its id on the
 * strip, or several beats and lulls gathered with Shift, named by theirs
 * together. */
export type ToolPanelStripItem =
  | {
      /** How long it plays, in ms. */
      clipMs: number;
      id: string;
      /** Whether a song plays without its singing. */
      isInstrumental: boolean;
      isLooped: boolean;
      isSong: boolean;
      kind: "audioClip";
      /** How far into its sound it starts, in ms. */
      offsetMs: number;
      /** The sound it plays, by its name in the library. */
      sound: string;
      /** How long its sound is, in ms. */
      soundMs: number;
      /** Its sound's title, or the name of the file it came from. */
      title: string;
      /** How much louder or quieter than its default level, in dB. */
      volumeDb: number;
    }
  | { beat: BeatWhat; id: string; kind: "beat" }
  | { id: string; kind: "lull"; play: LullPlay }
  | {
      /** The recording, whose voice notes the panel plays itself. */
      artifactId: number;
      /** The kind's colour, `#rrggbb`. */
      color: string;
      id: string;
      /** Which moment it is, counting from zero in the order placed. */
      index: number;
      kind: "moment";
      /** The kind's name. */
      name: string;
      note: {
        /** Peak levels from 0 to 1, evenly along it. */
        peaks: number[];
        transcript: NoteTranscript | null;
      } | null;
    }
  | {
      id: string;
      kind: "range";
      /** Where the range plays in the edited video, in milliseconds. */
      played: { endMs: number; startMs: number };
    }
  | {
      /** How many beats and lulls are chosen, two or more. */
      count: number;
      id: string;
      kind: "stripGroup";
    };

/** A track the export and the preview hold or leave out: one recorded, or
 * the songs or sound effects laid under the recording. */
export type ToolPanelTrack = {
  /** The track's id, as the editor selects it. */
  id: string;
  isEnabled: boolean;
  /** Its own name, such as Screen or Microphone. */
  label: string;
  /** Laid under the recording rather than recorded, so it never counts as
   * the last track left to export. */
  isLaidUnder?: boolean;
};

/** The songs or the sound effects on the strip, and whether they are
 * heard. */
export type ToolPanelSounds = {
  isHeard: boolean;
  kind: AudioClipKind;
};
