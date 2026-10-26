// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { CSSProperties } from "react";

// What the browser knows about a project: what it is, and what the app
// already knows happened in it. A recording's cursor and key files give
// every click, typing run, shortcut and app switch, its transcript what was
// said, and the editor where it was left; so a project can be browsed by
// what happened in it, not only by its name and a still.

/** A place on the recorded screen, in percent of it. */
export type Box = { h: number; w: number; x: number; y: number };

/** A box placed on the recorded screen, which is drawn left to right, so
 * its start is the screen's left. */
export const place = ({ h, w, x, y }: Box): CSSProperties => ({
  height: `${String(h)}%`,
  insetInlineStart: `${String(x)}%`,
  top: `${String(y)}%`,
  width: `${String(w)}%`,
});

export type Beat = {
  at: number;
  kind: "click" | "idle" | "keys" | "switch" | "type";
  label: string;
  seconds?: number;
};

export type Line = { end: number; start: number; text: string };

/** Where the work moved on, with what the step is about outlined. */
export type Step = { end: number; start: number; title: string; focus?: Box };

/** What the recording shows, which decides how its stills are drawn. */
export type Look =
  "audio" | "browser" | "camera" | "code" | "design" | "slides" | "walkthrough";

/** Where the project is in its life, as the editor and exports left it. */
export type Life =
  | { stage: "recorded" }
  | { leftAt: number; stage: "editing" }
  | {
      exportedAt: number;
      exportedTo: string;
      stage: "exported";
      /** Edited after the export, so the export is behind. */
      leftAt?: number;
    };

export type Project = {
  beats: Beat[];
  /** Seconds. */
  duration: number;
  editedAt: number;
  id: string;
  life: Life;
  lines: Line[];
  location: string;
  look: Look;
  /** Bytes on disk, everything in the project's folder. */
  size: number;
  steps: Step[];
  /** The colours a camera still is lit in. */
  tint: [string, string];
  title: string;
  /** Takes of the same thing, recorded one after another. */
  take?: { group: string; number: number };
};

export const HOUR = 60 * 60 * 1_000;
export const DAY = 24 * HOUR;
export const MB = 1_000_000;

/** A moment relative to now, so the story reads the same whenever opened. */
export const ago = (days: number, hours = 0) =>
  Date.now() - days * DAY - hours * HOUR;

/** The step a moment falls in. */
export const stepAt = (project: Project, at: number) =>
  Math.max(
    0,
    project.steps.findIndex(({ end, start }) => at >= start && at < end),
  );

/** The line being said at a moment, if any. */
export const lineAt = (project: Project, at: number) =>
  project.lines.find(({ end, start }) => at >= start && at < end);

/** Where the editor was left, if it was. */
export const leftAt = ({ life }: Project) =>
  life.stage === "recorded" ? undefined : life.leftAt;

/** Minutes and seconds. */
export const mmss = (seconds: number) =>
  `${String(Math.floor(seconds / 60))}:${String(Math.floor(seconds % 60)).padStart(2, "0")}`;

export const bytes = (size: number) =>
  size >= 1_000 * MB
    ? `${(size / (1_000 * MB)).toFixed(1)} GB`
    : `${String(Math.round(size / MB))} MB`;

/** How long ago, as a person says it. */
export const since = (at: number) => {
  const minutes = Math.round((Date.now() - at) / 60_000);
  if (minutes < 60) return `${String(minutes)} min ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${String(hours)} h ago`;
  const days = Math.round(hours / 24);
  if (days === 1) return "yesterday";
  if (days < 30) return `${String(days)} days ago`;
  const months = Math.round(days / 30);
  return months < 12 ? `${String(months)} months ago` : "over a year ago";
};

/** Where the project is, in a line: what was last done to it. */
export const lifeLine = (project: Project) => {
  const { life } = project;
  if (life.stage === "recorded") return "Not edited yet";
  const step = project.steps[stepAt(project, life.leftAt ?? 0)]?.title ?? "";
  if (life.stage === "editing") return `Left at ${mmss(life.leftAt)} · ${step}`;
  if (life.leftAt !== undefined)
    return `Changed since export to ${life.exportedTo}`;
  return `Exported to ${life.exportedTo} ${since(life.exportedAt)}`;
};

/** Exported a while ago and not touched since: its original can go and
 * the export stay. */
export const isReclaimable = (project: Project) =>
  project.life.stage === "exported" &&
  project.life.leftAt === undefined &&
  Date.now() - project.editedAt > 30 * DAY;

/** Newest first, in the groups a person thinks of them in. */
export const byDay = (projects: Project[]) => {
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const labelFor = (at: number) => {
    if (at >= today.getTime()) return "Today";
    if (at >= today.getTime() - DAY) return "Yesterday";
    if (at >= today.getTime() - 7 * DAY) return "This week";
    return "Earlier";
  };
  const groups: { label: string; projects: Project[] }[] = [];
  for (const project of [...projects].sort((a, b) => b.editedAt - a.editedAt)) {
    const label = labelFor(project.editedAt);
    const last = groups[groups.length - 1] as
      (typeof groups)[number] | undefined;
    if (last?.label === label) last.projects.push(project);
    else groups.push({ label, projects: [project] });
  }
  return groups;
};
