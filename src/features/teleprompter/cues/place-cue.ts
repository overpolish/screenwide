// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

/** A cue, by id, and the word it is on, by where the word starts. */
export type CuePlace = { at: number; id: string };

/** `cues` with each of `moved` at its word. */
export function movedTo<T extends CuePlace>(
  cues: readonly T[],
  moved: readonly CuePlace[],
): T[] {
  const words = new Map(moved.map(({ at, id }) => [id, at]));
  return cues.map((cue) => {
    const to = words.get(cue.id) ?? cue.at;
    return to === cue.at ? cue : { ...cue, at: to };
  });
}
