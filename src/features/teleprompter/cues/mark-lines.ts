// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

/**
 * The line each cue's mark is drawn level with, given the line its word is
 * on, `own`, with the cues in the order of their words and the lines
 * numbered from 0 to `lineCount` down the page. A line shows one mark, so
 * every mark can be seen and pressed. The first cue on a line keeps it;
 * each other goes to the first line after its own that no cue's word is on
 * and no mark has taken, or failing one, the nearest such line before it.
 * Where every line is taken it stays on its own, over another.
 */
export function markLines(own: readonly number[], lineCount: number) {
  const taken = new Set(own);
  const kept = new Set<number>();
  const free = (line: number) =>
    line >= 0 && line < lineCount && !taken.has(line);
  return own.map((line) => {
    if (!kept.has(line)) {
      kept.add(line);
      return line;
    }
    let to = line + 1;
    while (to < lineCount && !free(to)) to += 1;
    if (!free(to)) {
      to = line - 1;
      while (to >= 0 && !free(to)) to -= 1;
    }
    if (!free(to)) return line;
    taken.add(to);
    return to;
  });
}
