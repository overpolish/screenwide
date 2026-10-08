// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ProjectKind, ProjectSummary } from "./types";

export type KindFilter = "all" | ProjectKind;

export type ProjectGroup = { label: string; projects: ProjectSummary[] };

/** The projects whose kind the filter names and whose title holds the query,
 * matched without regard to case. A project of unknown kind shows only under
 * "all". */
export function filterProjects(
  projects: readonly ProjectSummary[],
  query: string,
  kind: KindFilter,
) {
  const needle = query.trim().toLocaleLowerCase();
  return projects.filter(
    (project) =>
      (kind === "all" || project.kind === kind) &&
      project.title.toLocaleLowerCase().includes(needle),
  );
}

const DAY_MS = 24 * 60 * 60 * 1_000;

/** How long a project in Recently Deleted has before it goes to the Trash,
 * in whole days rounded up, as Photos counts them: one that goes in an hour
 * still has a day. */
export function timeLeft(expiresMs: number, nowMs: number) {
  const days = Math.max(1, Math.ceil((expiresMs - nowMs) / DAY_MS));
  return days === 1 ? "1 day left" : `${String(days)} days left`;
}

const monthName = new Intl.DateTimeFormat(undefined, { month: "long" });
const monthAndYear = new Intl.DateTimeFormat(undefined, {
  month: "long",
  year: "numeric",
});

/**
 * Newest first, in the groups Finder sorts by date into: Today, Yesterday,
 * Previous 7 Days and Previous 30 Days, then one group per calendar month,
 * named without the year while it is this year's. Days start at local
 * midnight, so a recording made at 23:59 is yesterday's a minute later. A
 * project with no edit time comes last, under its own heading.
 */
export function groupByDate(
  projects: readonly ProjectSummary[],
  now: Date,
): ProjectGroup[] {
  // Built from the calendar date rather than by subtracting days in
  // milliseconds, so a daylight-saving change does not move midnight.
  const midnight = (daysAgo: number) =>
    new Date(
      now.getFullYear(),
      now.getMonth(),
      now.getDate() - daysAgo,
    ).getTime();
  const labelFor = (modifiedMs: number | null) => {
    if (modifiedMs === null) return "Unknown Date";
    if (modifiedMs >= midnight(0)) return "Today";
    if (modifiedMs >= midnight(1)) return "Yesterday";
    if (modifiedMs >= midnight(7)) return "Previous 7 Days";
    if (modifiedMs >= midnight(30)) return "Previous 30 Days";
    const date = new Date(modifiedMs);
    return date.getFullYear() === now.getFullYear()
      ? monthName.format(date)
      : monthAndYear.format(date);
  };

  const sorted = [...projects].sort(
    (a, b) => (b.modifiedMs ?? -1) - (a.modifiedMs ?? -1),
  );
  const groups: ProjectGroup[] = [];
  for (const project of sorted) {
    const label = labelFor(project.modifiedMs);
    const last = groups[groups.length - 1] as ProjectGroup | undefined;
    if (last?.label === label) last.projects.push(project);
    else groups.push({ label, projects: [project] });
  }
  return groups;
}
