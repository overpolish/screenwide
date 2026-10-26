// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ago, MB, type Project } from "./projects-model";

// The library's older projects: a talk, a podcast, three takes of one
// tutorial, and a bug report long since exported, whose original only takes
// up disk.

const settingsTake = (number: number): Project => ({
  beats: [
    { at: 4, kind: "click", label: "Settings" },
    { at: 14, kind: "click", label: "Tools" },
    { at: 26 + number * 3, kind: "click", label: "Glide" },
    ...(number === 2
      ? [
          {
            at: 40,
            kind: "idle" as const,
            label: "Nothing happens",
            seconds: 12,
          },
        ]
      : []),
  ],
  duration: 60 + number * 6,
  editedAt: ago(8, 4 - number),
  id: `settings-${String(number)}`,
  life: { stage: "recorded" },
  lines: [
    { end: 12, start: 1, text: "Settings now show what they do." },
    {
      end: 40,
      start: 20,
      text:
        number === 3
          ? "Each tool plays its effect right here."
          : "Each tool, erm, shows its effect.",
    },
  ],
  location: "Tutorials",
  look: "design",
  size: (80 + number * 9) * MB,
  steps: [
    {
      end: 20,
      focus: { h: 7, w: 18, x: 3, y: 18 },
      start: 0,
      title: "Open Settings",
    },
    {
      end: 60 + number * 6,
      focus: { h: 34, w: 70, x: 26, y: 18 },
      start: 20,
      title: "Try Glide",
    },
  ],
  take: { group: "settings", number },
  tint: ["#cdeee0", "#9fd5d1"],
  title: "Settings redesign",
});

export const OLDER_PROJECTS: Project[] = [
  {
    beats: [
      { at: 3, kind: "switch", label: "Keynote" },
      { at: 30, kind: "keys", label: "→" },
      { at: 92, kind: "keys", label: "→" },
      { at: 160, kind: "keys", label: "→" },
      { at: 240, kind: "keys", label: "→" },
      { at: 300, kind: "idle", label: "Nothing happens", seconds: 22 },
    ],
    duration: 372,
    editedAt: ago(4),
    id: "roadmap",
    life: { leftAt: 160, stage: "editing" },
    lines: [
      { end: 24, start: 6, text: "This is the plan for the next quarter." },
      { end: 88, start: 34, text: "First, recording gets a replay buffer." },
      { end: 150, start: 96, text: "Then the editor learns about steps." },
      { end: 230, start: 164, text: "Export gets presets for every app." },
      { end: 290, start: 244, text: "And we ship on Windows." },
    ],
    location: "Tutorials",
    look: "slides",
    size: 540 * MB,
    steps: [
      { end: 30, start: 0, title: "The plan" },
      { end: 92, start: 30, title: "Replay buffer" },
      { end: 160, start: 92, title: "Steps" },
      { end: 240, start: 160, title: "Export presets" },
      { end: 372, start: 240, title: "Windows" },
    ],
    tint: ["#fde68a", "#f59e0b"],
    title: "Q3 roadmap",
  },
  {
    beats: [],
    duration: 45,
    editedAt: ago(5),
    id: "podcast",
    life: { exportedAt: ago(5), exportedTo: "Music", stage: "exported" },
    lines: [
      { end: 12, start: 1, text: "Welcome to Overpolish, the podcast." },
      { end: 30, start: 15, text: "Today: why screen recordings are boring." },
      { end: 44, start: 33, text: "And what we're doing about it." },
    ],
    location: "Screenwide",
    look: "audio",
    size: 4 * MB,
    steps: [
      { end: 15, start: 0, title: "Welcome" },
      { end: 45, start: 15, title: "Today's topic" },
    ],
    tint: ["#2b2d42", "#111111"],
    title: "Podcast intro",
  },
  settingsTake(1),
  settingsTake(2),
  settingsTake(3),
  {
    beats: [
      { at: 5, kind: "switch", label: "Safari" },
      { at: 12, kind: "click", label: "Sign up" },
      { at: 20, kind: "idle", label: "Nothing happens", seconds: 30 },
    ],
    duration: 71,
    editedAt: ago(410),
    id: "old-bug",
    life: { exportedAt: ago(400), exportedTo: "Desktop", stage: "exported" },
    lines: [{ end: 11, start: 2, text: "The sign up button does nothing." }],
    location: "Screenwide",
    look: "browser",
    size: 1_840 * MB,
    steps: [
      { end: 20, start: 0, title: "Open sign up" },
      {
        end: 71,
        focus: { h: 8, w: 18, x: 41, y: 74 },
        start: 20,
        title: "Nothing happens",
      },
    ],
    tint: ["#f5f5f4", "#d6d3d1"],
    title: "Sign up bug report",
  },
  {
    beats: [
      { at: 8, kind: "switch", label: "Safari" },
      { at: 30, kind: "click", label: "Download" },
      { at: 50, kind: "idle", label: "Nothing happens", seconds: 40 },
    ],
    duration: 112,
    editedAt: ago(220),
    id: "launch",
    life: { exportedAt: ago(215), exportedTo: "YouTube", stage: "exported" },
    lines: [{ end: 20, start: 4, text: "Screenwide is out today." }],
    location: "Client work",
    look: "browser",
    size: 960 * MB,
    steps: [
      { end: 50, start: 0, title: "The launch page" },
      { end: 112, start: 50, title: "Download" },
    ],
    tint: ["#f5f5f4", "#d6d3d1"],
    title: "Launch teaser",
  },
];
