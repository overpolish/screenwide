// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  projectReading,
  type ProjectReading,
} from "./activity/project-reading";
import { projects } from "./project-browser-fixtures";

import type { ProjectActivity } from "./types";

// What happened in the sample recordings, read the way the app reads a real
// one: what was said, clicked, typed and flagged, and where the picture went
// new. Times are in recorded seconds.

type Said = [at: number, text: string];

/** A transcript's words, spread over each line at a speaking pace. */
const wordsOf = (lines: Said[]) =>
  lines.flatMap(([at, text]) =>
    text.split(" ").map((word, index) => ({
      endMs: Math.round((at + index * 0.32 + 0.28) * 1000),
      probability: 1,
      startMs: Math.round((at + index * 0.32) * 1000),
      text: word,
    })),
  );

const activity = ({
  clicks = [],
  duration,
  moments = [],
  said = [],
  screens = [],
  typing = [],
}: {
  duration: number;
  clicks?: number[];
  moments?: [at: number, name: string, color: string, note?: string][];
  said?: Said[];
  screens?: number[];
  typing?: number[];
}): ProjectActivity => ({
  activity: {
    clicks: clicks.map((at) => ({ count: 1, ms: at * 1000 })),
    picture: screens.map((at) => ({
      endMs: (at + 0.6) * 1000,
      startMs: at * 1000,
      state: "new" as const,
    })),
    typing: typing.map((at) => at * 1000),
  },
  durationMs: duration * 1000,
  edit: null,
  moments: moments.map(([at, name, color, note], index) => ({
    color,
    index,
    name,
    note: note ?? null,
    sourceMs: at * 1000,
  })),
  words: wordsOf(said),
});

const RED = "#e5484d";
const BLUE = "#3e63dd";

const activities: Record<string, ProjectActivity> = {
  [projects[0].file]: activity({
    clicks: [9, 14, 31, 33, 52, 70],
    duration: 94,
    moments: [[60, "Highlight", RED, "Use this bit for the launch post"]],
    said: [
      [2, "Here is the new dashboard."],
      [18, "Open settings and turn on the pricing table."],
      [40, "Then share it with your team."],
      [76, "That is all it takes."],
    ],
    screens: [8, 30, 51],
    typing: [22, 22.5, 23, 23.4, 24],
  }),
  [projects[1].file]: activity({
    clicks: [20, 64, 130, 202, 330, 410, 515],
    duration: 612,
    moments: [
      [140, "Mistake", RED],
      [420, "Highlight", BLUE, "Pricing questions come up here"],
    ],
    said: [
      [4, "Welcome to the onboarding walkthrough."],
      [70, "First, create a workspace for your team."],
      [210, "Invite people by email or with a link."],
      [340, "The pricing page explains every plan."],
      [520, "Run npm test before you ship anything."],
    ],
    screens: [60, 200, 325, 500],
    typing: [80, 81, 82, 215, 216, 525, 526, 527],
  }),
  [projects[2].file]: activity({
    duration: 245,
    said: [
      [3, "Hi everyone, here is this week's update."],
      [60, "We shipped the new pricing page on Tuesday."],
      [150, "Next week we look at onboarding again."],
    ],
  }),
  [projects[3].file]: activity({
    clicks: [5, 12, 26, 40, 55],
    duration: 71,
    said: [[30, "The settings window now groups its rows."]],
    screens: [4, 25, 54],
    typing: [14, 14.4, 15],
  }),
  [projects[4].file]: activity({
    duration: 45,
    said: [
      [1, "Welcome back to the show."],
      [20, "Today we talk about pricing for small teams."],
    ],
  }),
  [projects[7].file]: activity({
    clicks: [3, 9, 20],
    duration: 31,
    screens: [2, 18],
  }),
  [projects[8].file]: activity({
    clicks: [2, 6, 11],
    duration: 18,
    said: [[1, "The export button does nothing on Safari."]],
    typing: [8, 8.4],
  }),
};

/** Each sample recording's reading, as the window passes them to the
 * browser; a screenshot has none. */
export const readings: Record<string, ProjectReading | null> =
  Object.fromEntries(
    projects.map(({ file, kind }) => [
      file,
      kind === "screenshot"
        ? null
        : file in activities
          ? projectReading(activities[file])
          : null,
    ]),
  );
