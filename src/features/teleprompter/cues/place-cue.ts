// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { type ScriptText } from "../script-words";

import { wordAt } from "./cue-anchors";

/** A cue, by id, and the word it is on, by where the word starts. */
export type CuePlace = { at: number; id: string };

/** The line the word with index `index` is laid out on, as a number growing
 * down the page. */
export type LineOfWord = (index: number) => number;

/**
 * `cues` with the cue `move.id` moved to the word starting `move.at`. A line
 * shows one cue's mark, so a cue on the line it is dropped on moves down to
 * the first word of the next line with no cue, where its move can be seen;
 * one with no such line after it stays. The others stay where they are. As
 * laid out by `lineOf`; without it each word stands as a line of its own.
 */
export function placeCue<T extends CuePlace>(
  cues: readonly T[],
  { at, id }: CuePlace,
  {
    lineOf = (index) => index,
    script,
  }: { script: ScriptText; lineOf?: LineOfWord },
): T[] {
  if (!cues.some((cue) => cue.id === id)) return [...cues];
  const lineAt = (start: number) => lineOf(wordAt(script, start));
  const others = cues.filter((cue) => cue.id !== id);
  const dropped = lineAt(at);
  const taken = new Set([dropped, ...others.map((cue) => lineAt(cue.at))]);
  const placed = new Map<string, number>([[id, at]]);
  const pushed = others
    .filter((cue) => lineAt(cue.at) === dropped)
    .sort((first, second) => first.at - second.at);
  for (const cue of pushed) {
    const from = lineAt(cue.at);
    const to = script.starts.findIndex((_, index) => {
      const line = lineOf(index);
      return line > from && !taken.has(line);
    });
    if (to < 0) continue;
    taken.add(lineOf(to));
    placed.set(cue.id, script.starts[to]);
  }
  return movedTo(
    cues,
    [...placed].map(([moved, to]) => ({ at: to, id: moved })),
  );
}

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
