// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { findIn } from "../../../lib/found-text";

import type { DateQuery } from "./date-query";
import type { DateRange } from "./date-range";
import type { ProjectReading } from "../activity/project-reading";
import type { ProjectSummary } from "../types";

// Search that reads inside recordings. A name is one thing to find a project
// by; what was said in it, the moments flagged while recording and what
// their voice notes say are the rest, and each is a moment that can be gone
// to rather than a file to open and hunt through. Matched without regard to
// case, in the order the projects are listed, each project's finds in the
// order they happen.

export type FindKind = "moment" | "note" | "said";

export type Find = {
  /** In recorded seconds. */
  at: number;
  kind: FindKind;
  /** Where the query falls in `text`. */
  match: [number, number];
  text: string;
};

export type SearchResult = {
  finds: Find[];
  project: ProjectSummary;
};

function searchProjects(
  projects: readonly ProjectSummary[],
  readings: Readonly<Record<string, ProjectReading | null>>,
  query: string,
): SearchResult[] {
  const needle = query.trim();
  if (needle === "") return [];
  return projects.flatMap((project) => {
    const reading = readings[project.file];
    const finds: Find[] = [];
    for (const line of reading?.lines ?? []) {
      const match = findIn(line.text, needle);
      // Said at about where the words fall in the line.
      if (match)
        finds.push({
          at:
            line.start +
            ((line.end - line.start) * match[0]) /
              Math.max(1, line.text.length),
          kind: "said",
          match,
          text: line.text,
        });
    }
    for (const moment of reading?.moments ?? []) {
      const named = findIn(moment.name, needle);
      const noted = moment.note === null ? null : findIn(moment.note, needle);
      if (noted && moment.note !== null)
        finds.push({
          at: moment.at,
          kind: "note",
          match: noted,
          text: moment.note,
        });
      else if (named)
        finds.push({
          at: moment.at,
          kind: "moment",
          match: named,
          text: moment.name,
        });
    }
    finds.sort((one, other) => one.at - other.at);
    return finds.length > 0 || findIn(project.title, needle)
      ? [{ finds, project }]
      : [];
  });
}

export type ProjectSearch = {
  /** The date the query was read as, or null when it is words alone. */
  date: DateRange | null;
  /** The projects found, in the order listed. */
  projects: ProjectSummary[];
  /** What the words were found in, project by project; null for a date
   * alone, whose projects show as the list does. */
  results: SearchResult[] | null;
  /** The words searched for, marked where they are found; "" for none. */
  words: string;
};

/**
 * A search by words, by when the projects were recorded (`createdMs`), or
 * both: "monday pricing" looks for "pricing" in Monday's recordings. Words
 * that read as a date but are found as typed are taken as words: a project
 * named "May launch" is found by "may launch" whenever it was recorded. A
 * date alone that finds nothing falls back to the projects named with it,
 * so "may" still finds "May launch" when nothing was recorded in May, and
 * "yesterday" with nothing recorded yesterday says there are no results
 * rather than listing where the word was said.
 */
export function searchByQuery(
  projects: readonly ProjectSummary[],
  readings: Readonly<Record<string, ProjectReading | null>>,
  query: {
    /** The query's date, if it names one. */
    date: DateQuery | null;
    text: string;
  },
): ProjectSearch {
  const { date, text } = query;
  const literal = searchProjects(projects, readings, text);
  const asWords: ProjectSearch = {
    date: null,
    projects: literal.map(({ project }) => project),
    results: literal,
    words: text.trim(),
  };
  if (!date) return asWords;
  const recorded = projects.filter(
    ({ createdMs }) =>
      createdMs !== null && createdMs >= date.start && createdMs < date.end,
  );
  if (date.words === "") {
    const named = literal.filter(({ project }) =>
      findIn(project.title, text.trim()),
    );
    return recorded.length === 0 && named.length > 0
      ? {
          ...asWords,
          projects: named.map(({ project }) => project),
          results: named,
        }
      : { date, projects: recorded, results: null, words: "" };
  }
  if (literal.length > 0) return asWords;
  const results = searchProjects(recorded, readings, date.words);
  return {
    date,
    projects: results.map(({ project }) => project),
    results,
    words: date.words,
  };
}
