// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { BeatWhat } from "../island/beat/beat-island";
import type { LullPlay } from "../island/lull/lull-island";
import type { NoteTranscript } from "../timeline/moments/recording-moments";

/** What is chosen on the strip, where it has an island: one beat, lull,
 * moment or drawn range, named by its id on the strip, or several beats
 * and lulls gathered with Shift, named by theirs together. */
export type ToolPanelStripItem =
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

/** A recorded track the export and the preview hold or leave out. */
export type ToolPanelTrack = {
  /** The track's id, as the editor selects it. */
  id: string;
  isEnabled: boolean;
  /** Its own name, such as Screen or Microphone. */
  label: string;
};
