// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { type ScriptText } from "../script-words";

// A cue is kept by where its word starts in the script. An edit moves it
// with its word: text typed or taken out before the word shifts it, an edit
// after it leaves it be, and one that takes its word away moves it to the
// word that comes next.

type Anchored = { at: number };

/** The start of the word `position` falls in, or of the next one; the last
 * word's where none follows; 0 in a script with no words. */
export const wordStartAt = (script: ScriptText, position: number) => {
  const index = wordAt(script, position);
  return index < 0 ? 0 : script.starts[index];
};

/** The index of the word `position` falls in, or of the next one; the last
 * word's where none follows; -1 in a script with no words. */
export function wordAt(script: ScriptText, position: number) {
  const { pieces, starts } = script;
  if (starts.length === 0) return -1;
  for (const piece of pieces) {
    if (piece.kind !== "word") continue;
    const start = starts[piece.index];
    if (position < start + piece.text.length) return piece.index;
  }
  return starts.length - 1;
}

/** Each of `cues` where its word is once `before` is edited into `after`,
 * `script` being `after` as it is read. */
export function cuesAfterEdit<T extends Anchored>(
  {
    after,
    before,
    script,
  }: { after: string; before: string; script: ScriptText },
  cues: readonly T[],
): T[] {
  let prefix = 0;
  const shortest = Math.min(before.length, after.length);
  while (prefix < shortest && before[prefix] === after[prefix]) prefix += 1;
  let suffix = 0;
  while (
    suffix < shortest - prefix &&
    before[before.length - 1 - suffix] === after[after.length - 1 - suffix]
  )
    suffix += 1;
  const editedEnd = before.length - suffix;
  const shift = after.length - before.length;
  return cues.map((cue) => {
    const moved =
      cue.at < prefix
        ? cue.at
        : cue.at >= editedEnd
          ? cue.at + shift
          : prefix + (after.length - suffix - prefix);
    const at = wordStartAt(script, moved);
    return at === cue.at ? cue : { ...cue, at };
  });
}
