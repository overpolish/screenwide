// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Project } from "./projects-model";

// Telling takes of one thing apart by what the recording already knows:
// how long nothing happened, and whether a filler word was said.

/** What a take has against it. */
export const flaws = (take: Project) => [
  ...take.beats
    .filter(({ kind }) => kind === "idle")
    .map(({ seconds }) => `${String(seconds ?? 0)} s pause`),
  ...(take.lines.some(({ text }) => /\berm\b/u.test(text)) ? ["Says erm"] : []),
];

/** The take with least against it, the latest of those tied. */
export const bestTake = (takes: Project[]) =>
  [...takes].sort(
    (first, second) =>
      flaws(first).length - flaws(second).length ||
      (second.take?.number ?? 0) - (first.take?.number ?? 0),
  )[0];
