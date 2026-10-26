// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe } from "../ideas/action-script-lines";

import { type Beat, type Project } from "./projects-model";

// Search that reads inside recordings. A name is one thing to find a
// project by; what was said in it, what was clicked and typed, which app it
// moved to and what its steps are about are the rest, and each is a moment
// that can be gone to rather than a file to open and hunt through.

export type Hit = {
  at: number;
  kind: Beat["kind"] | "said" | "step";
  /** Where the query falls in `text`. */
  match: [number, number];
  text: string;
};

export type Result = {
  /** The name matched, which needs no moment. */
  byTitle: boolean;
  hits: Hit[];
  project: Project;
};

const find = (text: string, needle: string): [number, number] | null => {
  const index = text.toLocaleLowerCase().indexOf(needle);
  return index === -1 ? null : [index, index + needle.length];
};

export function searchMoments(projects: Project[], query: string): Result[] {
  const needle = query.trim().toLocaleLowerCase();
  if (needle === "") return [];
  return projects.flatMap((project) => {
    const hits: Hit[] = [];
    for (const line of project.lines) {
      const match = find(line.text, needle);
      // Said at about where the word falls in the sentence.
      if (match)
        hits.push({
          at:
            line.start +
            ((line.end - line.start) * match[0]) /
              Math.max(1, line.text.length),
          kind: "said",
          match,
          text: line.text,
        });
    }
    for (const beat of project.beats) {
      if (beat.kind === "idle") continue;
      const text = describe(beat);
      const match = find(text, needle);
      if (match) hits.push({ at: beat.at, kind: beat.kind, match, text });
    }
    for (const step of project.steps) {
      const match = find(step.title, needle);
      if (match)
        hits.push({ at: step.start, kind: "step", match, text: step.title });
    }
    hits.sort((first, second) => first.at - second.at);
    const byTitle = find(project.title, needle) !== null;
    return hits.length > 0 || byTitle ? [{ byTitle, hits, project }] : [];
  });
}

/** Queries worth trying, from what the library holds. */
export const SUGGESTIONS = ["Safari", "pricing", "npm test", "Create"];
