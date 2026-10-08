// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { filterProjects, groupByDate, timeLeft } from "./project-list";

import type { ProjectSummary } from "./types";

const project = (
  title: string,
  overrides: Partial<ProjectSummary> = {},
): ProjectSummary => ({
  available: true,
  durationMs: null,
  expiresMs: null,
  file: `/projects/${title}.screenwide`,
  kind: "screen",
  modifiedMs: null,
  replay: false,
  sizeBytes: null,
  title,
  tracks: null,
  ...overrides,
});

const at = (...parts: [number, number, number, number?, number?]) =>
  new Date(...parts).getTime();

// Wednesday 8 October 2026, mid-afternoon local time.
const now = new Date(2026, 9, 8, 15, 30);

const labels = (projects: ProjectSummary[]) =>
  groupByDate(projects, now).map(({ label, projects: members }) => [
    label,
    members.map(({ title }) => title),
  ]);

describe("groupByDate", () => {
  it("splits days at local midnight", () => {
    expect(
      labels([
        project("just after midnight", { modifiedMs: at(2026, 9, 8, 0, 0) }),
        project("a minute before", { modifiedMs: at(2026, 9, 7, 23, 59) }),
      ]),
    ).toEqual([
      ["Today", ["just after midnight"]],
      ["Yesterday", ["a minute before"]],
    ]);
  });

  it("bounds the rolling groups at 7 and 30 days before today", () => {
    expect(
      labels([
        project("7 days", { modifiedMs: at(2026, 9, 1) }),
        project("8 days", { modifiedMs: at(2026, 8, 30, 23) }),
        project("30 days", { modifiedMs: at(2026, 8, 8) }),
        project("31 days", { modifiedMs: at(2026, 8, 7, 23) }),
      ]),
    ).toEqual([
      ["Previous 7 Days", ["7 days"]],
      ["Previous 30 Days", ["8 days", "30 days"]],
      [
        new Intl.DateTimeFormat(undefined, { month: "long" }).format(
          new Date(2026, 8, 7),
        ),
        ["31 days"],
      ],
    ]);
  });

  it("names the year only for months of earlier years", () => {
    const [label] = labels([
      project("last year", { modifiedMs: at(2025, 11, 31) }),
    ]);
    expect(label[0]).toBe(
      new Intl.DateTimeFormat(undefined, {
        month: "long",
        year: "numeric",
      }).format(new Date(2025, 11, 31)),
    );
  });

  it("orders newest first and puts undated projects last", () => {
    expect(
      labels([
        project("undated"),
        project("morning", { modifiedMs: at(2026, 9, 8, 9) }),
        project("afternoon", { modifiedMs: at(2026, 9, 8, 14) }),
      ]),
    ).toEqual([
      ["Today", ["afternoon", "morning"]],
      ["Unknown Date", ["undated"]],
    ]);
  });
});

describe("filterProjects", () => {
  const projects = [
    project("Product Demo"),
    project("Demo voiceover", { kind: "audio" }),
    project("Missing", { kind: null }),
  ];
  const titles = (list: ProjectSummary[]) => list.map(({ title }) => title);

  it("matches the query anywhere in the title, ignoring case", () => {
    expect(titles(filterProjects(projects, "  demo ", "all"))).toEqual([
      "Product Demo",
      "Demo voiceover",
    ]);
  });

  it("applies the kind and the query together", () => {
    expect(titles(filterProjects(projects, "demo", "audio"))).toEqual([
      "Demo voiceover",
    ]);
  });

  it("keeps a project of unknown kind only under all", () => {
    expect(titles(filterProjects(projects, "", "all"))).toContain("Missing");
    expect(titles(filterProjects(projects, "", "screen"))).toEqual([
      "Product Demo",
    ]);
  });
});

describe("timeLeft", () => {
  const day = 24 * 60 * 60 * 1_000;
  it("rounds part of a day up, so the last day reads as one left", () => {
    expect(timeLeft(30 * day, 0)).toBe("30 days left");
    expect(timeLeft(29 * day + 1, 0)).toBe("30 days left");
    expect(timeLeft(day, 0)).toBe("1 day left");
    expect(timeLeft(60 * 60 * 1_000, 0)).toBe("1 day left");
  });
});
