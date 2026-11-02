// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { appLocale, t } from "../../../i18n/i18n";

import type { ProjectKind, ProjectSummary } from "../types";

export type KindFilter = "all" | ProjectKind;

export type ProjectGroup = { label: string; projects: ProjectSummary[] };

/** The projects whose kind the filter names. A project of unknown kind
 * shows only under "all". */
export const filterByKind = (
  projects: readonly ProjectSummary[],
  kind: KindFilter,
) =>
  kind === "all" ? [...projects] : projects.filter((one) => one.kind === kind);

const DAY_MS = 24 * 60 * 60 * 1_000;

/** How long a project in Recently Deleted has before it goes to the Trash,
 * in whole days rounded up, as Photos counts them: one that goes in an hour
 * still has a day. */
export function timeLeft(expiresMs: number, nowMs: number) {
  const days = Math.max(1, Math.ceil((expiresMs - nowMs) / DAY_MS));
  return t("project-browser-days-left", { days });
}

// Built from the calendar date rather than by subtracting days in
// milliseconds, so a daylight-saving change does not move midnight.
const midnight = (now: Date, daysAgo: number) =>
  new Date(
    now.getFullYear(),
    now.getMonth(),
    now.getDate() - daysAgo,
  ).getTime();

/** How a card says when its project was recorded, as short as still tells
 * it apart: the weekday and time within the last six days, whose weekdays
 * are all different; the day and month for the rest of this year; the full
 * date before that. */
export function recordedStyle(
  recordedMs: number,
  now: Date,
): "date" | "day" | "weekday" {
  if (recordedMs >= midnight(now, 6)) return "weekday";
  return new Date(recordedMs).getFullYear() === now.getFullYear()
    ? "day"
    : "date";
}

/**
 * Newest first by when they were recorded, in the groups Finder sorts by
 * date into: Today, Yesterday and Previous 7 Days, then one group per
 * calendar month, named without the year while it is this year's. Days
 * start at local midnight, so a recording made at 23:59 is yesterday's a
 * minute later. A project with no recording time comes last, under its own
 * heading. Opening or editing a project leaves it where it is.
 */
export function groupByDate(
  projects: readonly ProjectSummary[],
  now: Date,
): ProjectGroup[] {
  const { formatLocale } = appLocale();
  const monthName = new Intl.DateTimeFormat(formatLocale, { month: "long" });
  const monthAndYear = new Intl.DateTimeFormat(formatLocale, {
    month: "long",
    year: "numeric",
  });
  const labelFor = (createdMs: number | null) => {
    if (createdMs === null) return t("project-browser-group-unknown");
    if (createdMs >= midnight(now, 0)) return t("project-browser-group-today");
    if (createdMs >= midnight(now, 1))
      return t("project-browser-group-yesterday");
    if (createdMs >= midnight(now, 7))
      return t("project-browser-group-previous-week");
    const date = new Date(createdMs);
    return date.getFullYear() === now.getFullYear()
      ? monthName.format(date)
      : monthAndYear.format(date);
  };

  const sorted = [...projects].sort(
    (a, b) => (b.createdMs ?? -1) - (a.createdMs ?? -1),
  );
  const groups: ProjectGroup[] = [];
  for (const project of sorted) {
    const label = labelFor(project.createdMs);
    const last = groups[groups.length - 1] as ProjectGroup | undefined;
    if (last?.label === label) last.projects.push(project);
    else groups.push({ label, projects: [project] });
  }
  return groups;
}
