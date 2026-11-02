// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { withoutIsolates as plain } from "../../../i18n/testing";

import {
  filterByKind,
  groupByDate,
  recordedStyle,
  timeLeft,
} from "./project-list";

import type { ProjectSummary } from "../types";

const project = (
  title: string,
  overrides: Partial<ProjectSummary> = {},
): ProjectSummary => ({
  available: true,
  createdMs: null,
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

// Tests run in the source language, en-US.
const labels = (projects: ProjectSummary[]) =>
  groupByDate(projects, now).map(({ label, projects: members }) => [
    label,
    members.map(({ title }) => title),
  ]);

describe("groupByDate", () => {
  it("splits days at local midnight", () => {
    expect(
      labels([
        project("just after midnight", { createdMs: at(2026, 9, 8, 0, 0) }),
        project("a minute before", { createdMs: at(2026, 9, 7, 23, 59) }),
      ]),
    ).toEqual([
      ["Today", ["just after midnight"]],
      ["Yesterday", ["a minute before"]],
    ]);
  });

  it("bounds the rolling group at 7 days before today, then goes by month", () => {
    expect(
      labels([
        project("7 days", { createdMs: at(2026, 9, 1) }),
        project("8 days", { createdMs: at(2026, 8, 30, 23) }),
        project("31 days", { createdMs: at(2026, 8, 7, 23) }),
      ]),
    ).toEqual([
      ["Previous 7 Days", ["7 days"]],
      [
        new Intl.DateTimeFormat("en-US", { month: "long" }).format(
          new Date(2026, 8, 30),
        ),
        ["8 days", "31 days"],
      ],
    ]);
  });

  it("names the year only for months of earlier years", () => {
    const [label] = labels([
      project("last year", { createdMs: at(2025, 11, 31) }),
    ]);
    expect(label[0]).toBe(
      new Intl.DateTimeFormat("en-US", {
        month: "long",
        year: "numeric",
      }).format(new Date(2025, 11, 31)),
    );
  });

  it("orders newest first and puts undated projects last", () => {
    expect(
      labels([
        project("undated"),
        project("morning", { createdMs: at(2026, 9, 8, 9) }),
        project("afternoon", { createdMs: at(2026, 9, 8, 14) }),
      ]),
    ).toEqual([
      ["Today", ["afternoon", "morning"]],
      ["Unknown Date", ["undated"]],
    ]);
  });

  it("goes by when a project was recorded, not when it was last edited", () => {
    expect(
      labels([
        project("opened today", {
          createdMs: at(2025, 11, 31),
          modifiedMs: at(2026, 9, 8, 15),
        }),
        project("recorded today", {
          createdMs: at(2026, 9, 8, 9),
          modifiedMs: at(2026, 9, 8, 9),
        }),
      ]).map(([, titles]) => titles),
    ).toEqual([["recorded today"], ["opened today"]]);
  });
});

describe("recordedStyle", () => {
  it("names the weekday within the last six days, whose weekdays differ", () => {
    expect(recordedStyle(at(2026, 9, 8, 9), now)).toBe("weekday");
    expect(recordedStyle(at(2026, 9, 2, 0, 0), now)).toBe("weekday");
    expect(recordedStyle(at(2026, 9, 1, 23, 59), now)).toBe("day");
  });

  it("gives the day and month this year and the full date before it", () => {
    expect(recordedStyle(at(2026, 0, 1), now)).toBe("day");
    expect(recordedStyle(at(2025, 11, 31, 23, 59), now)).toBe("date");
  });
});

describe("filterByKind", () => {
  const projects = [
    project("Product Demo"),
    project("Demo voiceover", { kind: "audio" }),
    project("Missing", { kind: null }),
  ];
  const titles = (list: ProjectSummary[]) => list.map(({ title }) => title);

  it("keeps the kind chosen", () => {
    expect(titles(filterByKind(projects, "audio"))).toEqual(["Demo voiceover"]);
  });

  it("keeps a project of unknown kind only under all", () => {
    expect(titles(filterByKind(projects, "all"))).toContain("Missing");
    expect(titles(filterByKind(projects, "screen"))).toEqual(["Product Demo"]);
  });
});

describe("timeLeft", () => {
  const day = 24 * 60 * 60 * 1_000;
  it("rounds part of a day up, so the last day reads as one left", () => {
    expect(plain(timeLeft(30 * day, 0))).toBe("30 days left");
    expect(plain(timeLeft(29 * day + 1, 0))).toBe("30 days left");
    expect(plain(timeLeft(day, 0))).toBe("1 day left");
    expect(plain(timeLeft(60 * 60 * 1_000, 0))).toBe("1 day left");
  });
});
