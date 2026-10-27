// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Beat, type Recording } from "../recording";

import { type EditorKind } from "./use-editor";

// The beats: whatever must play as it was. What was done comes first, so a
// flagged, loud or new moment inside an action is that action's. A line
// said while nothing else holds the time is a beat of its own, so a stretch
// of talk is never drawn as a spring; a line said over what was done stays
// under it, in the voice row.

const overlaps = (first: Beat, second: Beat) =>
  first.start < second.end && second.start < first.end;

export function beatsOf(
  recording: Recording,
  { isVoiced, kind }: { isVoiced: boolean; kind: EditorKind },
): Beat[] {
  if (kind === "screenshot") return [];
  if (kind === "audio") return recording.speechBeats;
  const cues = recording.cueBeats.filter(
    (cue) => !recording.actionBeats.some((action) => overlaps(action, cue)),
  );
  const doing = [...recording.actionBeats, ...joined(cues)];
  const talk = isVoiced
    ? recording.speechBeats.filter(
        (line) => !doing.some((beat) => overlaps(beat, line)),
      )
    : [];
  return [...doing, ...talk].sort(
    (first, second) => first.start - second.start,
  );
}

/** Cues that overlap, as one beat: the first's. */
function joined(cues: Beat[]): Beat[] {
  const sorted = [...cues].sort((first, second) => first.start - second.start);
  return sorted.reduce<Beat[]>((beats, cue) => {
    const last = beats[beats.length - 1] as Beat | undefined;
    if (last && cue.start <= last.end) {
      last.end = Math.max(last.end, cue.end);
      return beats;
    }
    return [...beats, { ...cue }];
  }, []);
}
