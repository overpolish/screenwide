// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { parseDateQuery } from "./date-query";
import { dateVocabulary } from "./date-reading";
import { searchByQuery } from "./project-search";

import type { ProjectReading } from "../activity/project-reading";
import type { ProjectSummary } from "../types";

// Monday 2 November 2026, mid-morning local time.
const now = new Date(2026, 10, 2, 10, 0);

const project = (title: string, createdMs: number | null): ProjectSummary => ({
  available: true,
  createdMs,
  durationMs: null,
  expiresMs: null,
  file: `/projects/${title}.screenwide`,
  kind: "screen",
  modifiedMs: null,
  replay: false,
  sizeBytes: null,
  title,
  tracks: null,
});

const projects = [
  project("Pricing walkthrough", new Date(2026, 10, 2, 0, 0).getTime()),
  project("Standup", new Date(2026, 10, 1, 23, 59).getTime()),
  project("Pricing review", new Date(2026, 10, 1, 15, 20).getTime()),
  project("May launch", new Date(2026, 9, 2, 9).getTime()),
  project("Unknown", null),
];

const reading = (said: string): ProjectReading =>
  ({ lines: [{ end: 4, start: 0, text: said }], moments: [] }) as never;

const readings = {
  [projects[1].file]: reading("We went over pricing for teams."),
};

const search = (text: string) =>
  searchByQuery(projects, readings, {
    date: parseDateQuery(text, now, dateVocabulary()),
    text,
  });

const titles = (found: { projects: ProjectSummary[] }) =>
  found.projects.map(({ title }) => title);

describe("searchByQuery", () => {
  it("lists a date alone's projects by when they were recorded", () => {
    const found = search("yesterday");
    expect(found.results).toBeNull();
    expect(found.words).toBe("");
    // A minute before midnight is yesterday's; midnight is today's.
    expect(titles(found)).toEqual(["Standup", "Pricing review"]);
    expect(titles(search("today"))).toEqual(["Pricing walkthrough"]);
  });

  it("looks for the words within the date", () => {
    const found = search("yesterday pricing");
    expect(found.date).not.toBeNull();
    expect(found.words).toBe("pricing");
    expect(titles(found)).toEqual(["Standup", "Pricing review"]);
    expect(found.results?.[0].finds).toHaveLength(1);
  });

  it("finds a project named like a date by its name", () => {
    const found = search("May launch");
    expect(found.date).toBeNull();
    expect(titles(found)).toEqual(["May launch"]);
  });

  it("falls back to the words when a date alone finds nothing", () => {
    const found = search("may");
    expect(found.date).toBeNull();
    expect(titles(found)).toEqual(["May launch"]);
  });

  it("finds nothing for a date with nothing recorded, though it is said", () => {
    const found = searchByQuery(
      projects,
      { [projects[1].file]: reading("See you on friday.") },
      { date: parseDateQuery("friday", now, dateVocabulary()), text: "friday" },
    );
    expect(found.date).not.toBeNull();
    expect(found.projects).toEqual([]);
  });

  it("searches words alone as before", () => {
    const found = search("pricing");
    expect(found.date).toBeNull();
    expect(titles(found)).toEqual([
      "Pricing walkthrough",
      "Standup",
      "Pricing review",
    ]);
  });
});
