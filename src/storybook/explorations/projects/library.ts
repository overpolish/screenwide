// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { STEPS as WALKTHROUGH_STEPS } from "../editor/editor-data";
import {
  ACTIONS,
  elementById,
  RECORDING_S,
  SEGMENTS,
} from "../ideas/ideas-data";
import { PROJECT_NAME } from "../shared/mock-data";

import { OLDER_PROJECTS } from "./library-older";
import { ago, MB, type Project } from "./projects-model";

// The library the browser shows. The walkthrough the editor exploration
// edits is one of its projects, with that exploration's own data, so the
// two read as one app; the rest are drawn to be as varied as a real
// library: a web page, code, a talk to camera, slides, sound, takes.

const walkthrough: Project = {
  beats: ACTIONS,
  duration: RECORDING_S,
  editedAt: ago(0, 1),
  id: "walkthrough",
  life: { leftAt: 20, stage: "editing" },
  lines: SEGMENTS.flatMap((segment) =>
    segment.kind === "speech"
      ? [{ end: segment.end, start: segment.start, text: segment.text }]
      : [],
  ),
  location: "Screenwide",
  look: "walkthrough",
  size: 148 * MB,
  steps: WALKTHROUGH_STEPS.map(({ element, end, start, title }) => ({
    end,
    focus: elementById(element)?.box,
    start,
    title,
  })),
  tint: ["#f6d5c3", "#c9d7f7"],
  title: PROJECT_NAME,
};

export const PROJECTS: Project[] = [
  walkthrough,
  {
    beats: [
      { at: 6, kind: "switch", label: "Safari" },
      { at: 11, kind: "type", label: "screenwide.app/pricing" },
      { at: 24, kind: "click", label: "Compare plans" },
      { at: 41, kind: "idle", label: "Nothing happens", seconds: 9 },
      { at: 58, kind: "click", label: "Team" },
      { at: 77, kind: "click", label: "Start free trial" },
      { at: 96, kind: "type", label: "dom@overpolish.com" },
      { at: 118, kind: "click", label: "Continue" },
    ],
    duration: 134,
    editedAt: ago(0, 3),
    id: "pricing",
    life: { exportedAt: ago(0, 2), exportedTo: "Desktop", stage: "exported" },
    lines: [
      { end: 9, start: 1, text: "Here's the new pricing page." },
      { end: 30, start: 20, text: "You can compare every plan side by side." },
      { end: 66, start: 54, text: "Most teams start on the Team plan." },
      { end: 90, start: 80, text: "The trial needs no card." },
      { end: 128, start: 112, text: "And checkout takes one step." },
    ],
    location: "Client work",
    look: "browser",
    size: 412 * MB,
    steps: [
      {
        end: 20,
        focus: { h: 7, w: 54, x: 23, y: 11 },
        start: 0,
        title: "Open the page",
      },
      {
        end: 70,
        focus: { h: 40, w: 26, x: 37, y: 42 },
        start: 20,
        title: "Compare plans",
      },
      {
        end: 110,
        focus: { h: 8, w: 18, x: 41, y: 74 },
        start: 70,
        title: "Start a trial",
      },
      { end: 134, start: 110, title: "Check out" },
    ],
    tint: ["#e0e7ff", "#fbcfe8"],
    title: "Pricing page tour",
  },
  {
    beats: [
      { at: 4, kind: "switch", label: "Terminal" },
      { at: 9, kind: "type", label: "npm run dev" },
      { at: 31, kind: "click", label: "Sign in" },
      { at: 52, kind: "switch", label: "Zed" },
      { at: 70, kind: "keys", label: "⌘P" },
      { at: 74, kind: "type", label: "session.ts" },
      { at: 96, kind: "idle", label: "Nothing happens", seconds: 14 },
      { at: 131, kind: "type", label: "expiresAt < now" },
      { at: 152, kind: "keys", label: "⌘S" },
      { at: 166, kind: "type", label: "npm test" },
    ],
    duration: 185,
    editedAt: ago(1, 2),
    id: "login-bug",
    life: { stage: "recorded" },
    lines: [
      { end: 22, start: 12, text: "Signing in drops you straight back out." },
      { end: 66, start: 54, text: "The session check looks wrong." },
      { end: 140, start: 124, text: "It compares the expiry the wrong way." },
      { end: 182, start: 168, text: "Tests pass, sign in stays." },
    ],
    location: "Screenwide",
    look: "code",
    size: 286 * MB,
    steps: [
      {
        end: 50,
        focus: { h: 22, w: 78, x: 22, y: 78 },
        start: 0,
        title: "Reproduce it",
      },
      {
        end: 120,
        focus: { h: 6, w: 15, x: 2, y: 31 },
        start: 50,
        title: "Find the check",
      },
      {
        end: 160,
        focus: { h: 6, w: 52, x: 26, y: 40 },
        start: 120,
        title: "Fix it",
      },
      {
        end: 185,
        focus: { h: 22, w: 78, x: 22, y: 78 },
        start: 160,
        title: "Run the tests",
      },
    ],
    tint: ["#1d2547", "#0b0f1e"],
    title: "Login bug",
  },
  {
    beats: [],
    duration: 245,
    editedAt: ago(1, 6),
    id: "weekly",
    life: {
      exportedAt: ago(3),
      exportedTo: "Movies",
      leftAt: 96,
      stage: "exported",
    },
    lines: [
      { end: 14, start: 2, text: "Morning everyone, quick weekly update." },
      { end: 70, start: 40, text: "Pricing shipped on Tuesday." },
      { end: 130, start: 100, text: "The login bug is fixed and in review." },
      { end: 230, start: 190, text: "Next week is all about the editor." },
    ],
    location: "Screenwide",
    look: "camera",
    size: 320 * MB,
    steps: [
      { end: 40, start: 0, title: "Hello" },
      { end: 100, start: 40, title: "Pricing" },
      { end: 190, start: 100, title: "Login bug" },
      { end: 245, start: 190, title: "Next week" },
    ],
    tint: ["#e8c9a8", "#5b4636"],
    title: "Weekly update",
  },
  ...OLDER_PROJECTS,
];
