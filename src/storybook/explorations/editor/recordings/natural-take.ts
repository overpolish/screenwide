// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Action } from "../../ideas/ideas-data";
import { MARK_S } from "../editor-data";
import { recordingOf, type SpokenLine } from "../recording";

// A three-minute walkthrough recorded in one take, the way people record:
// a slow start, a stray "um", exploring before choosing, waiting on a load,
// a phone checked mid-take, a change of mind about where to share, and a
// tail while reaching for stop. Same app and controls as the walkthrough,
// so the picture and the pointer have something to show.

const SPEECH: SpokenLine[] = [
  [1.6, 6, "Okay, so this is a quick look at projects in Screenwide."],
  [7.2, 11.8, "Everything you record ends up here, in the projects list."],
  [13, 17.4, "Recent is whatever you opened last, which is handy."],
  [18.4, 22.6, "And Shared is anything your team has sent over."],
  [29.8, 31.6, "Um, sorry."],
  [31.6, 35.4, "Let's start a brand new project."],
  [38.2, 43.4, "It asks for a name first, so I'll call this Launch video."],
  [47.8, 53.6, "Then you pick a template. There are three to choose from."],
  [
    55,
    60.8,
    "Blank is a clean page, Starter gives you a layout, and Gallery is for images.",
  ],
  [66.5, 70.2, "I think Starter suits this best."],
  [71.4, 74.6, "Then Create."],
  [80.4, 85.6, "There we go. The template has laid out the pages for you."],
  [88, 92.6, "Let me save it before I forget. Command S."],
  [103.8, 106.2, "Right, where was I."],
  [
    107.5,
    113.4,
    "Now I want to show the team, so I'll switch to Safari to grab the link.",
  ],
  [118.2, 121, "Actually, no, I can share it right here."],
  [122.4, 126.8, "Click Share in the top corner."],
  [
    131,
    137.2,
    "You can search for anyone on your team and send it straight to them.",
  ],
  [141.6, 145, "And that's sent."],
  [
    149.5,
    156,
    "Back in the projects list, you'll see it at the top under Recent.",
  ],
  [
    158,
    165.4,
    "So that's projects: make one, pick a template, and share it with your team.",
  ],
  [170.6, 174.2, "Thanks for watching."],
].map(([start, end, text]) => ({
  end: end as number,
  start: start as number,
  text: text as string,
}));

const ACTIONS: Action[] = [
  { at: 9.6, element: "projects", kind: "click", label: "Projects" },
  { at: 14.6, element: "recent", kind: "click", label: "Recent" },
  { at: 20.4, element: "shared", kind: "click", label: "Shared" },
  { at: 23.2, kind: "idle", label: "Nothing happens", seconds: 6.6 },
  { at: 34.2, element: "new", kind: "click", label: "New Project" },
  { at: 41, element: "name", kind: "click", label: "Project name" },
  { at: 42.6, element: "name", kind: "type", label: "Launch video" },
  { at: 47, kind: "keys", label: "⌘Z" },
  { at: 56.4, element: "blank", kind: "click", label: "Blank" },
  { at: 58.2, element: "starter", kind: "click", label: "Starter" },
  { at: 60, element: "gallery", kind: "click", label: "Gallery" },
  { at: 61.2, kind: "idle", label: "Nothing happens", seconds: 5.3 },
  { at: 68.9, element: "starter", kind: "click", label: "Starter" },
  { at: 73.1, element: "create", kind: "click", label: "Create" },
  { at: 91.4, kind: "keys", label: "⌘S" },
  { at: 93, kind: "idle", label: "Nothing happens", seconds: 10.8 },
  { at: 111.8, kind: "switch", label: "Safari" },
  { at: 120, kind: "switch", label: "Screenwide" },
  { at: 125.3, element: "share", kind: "click", label: "Share" },
  { at: 128.4, kind: "keys", label: "⌘C" },
  { at: 133.2, element: "search", kind: "click", label: "Search" },
  { at: 134.8, element: "search", kind: "type", label: "Maya" },
  { at: 143.4, element: "share", kind: "click", label: "Send" },
  { at: 147.5, kind: "keys", label: "⌘W" },
  { at: 151.6, element: "projects", kind: "click", label: "Projects" },
  { at: 154.2, element: "recent", kind: "click", label: "Recent" },
  { at: 166, kind: "idle", label: "Nothing happens", seconds: 4.6 },
  { at: 174.8, kind: "idle", label: "Nothing happens", seconds: 5.2 },
];

const markAt = (at: number) => ({ end: at + MARK_S, start: at });

export const NATURAL_TAKE = recordingOf({
  actions: ACTIONS,
  duration: 180,
  look: "screen",
  markTimes: { 1: markAt(68.9), 2: markAt(34.2), 3: markAt(125.3) },
  // Flagged while recording: the finished layout, the phone check to cut
  // later, and the share going out, which falls inside the Send click.
  moments: [84, 98, 144.6],
  picture: [
    // The new project sheet slides in.
    { end: 35.4, start: 34.8, state: "new" },
    { end: 36.6, start: 35.4, state: "moving" },
    // The project is made from the template, a bar filling as it goes.
    { end: 78.9, start: 73.6, state: "moving" },
    // Over to Safari and back.
    { end: 113, start: 112.4, state: "new" },
    { end: 121, start: 120.4, state: "new" },
    // The share sheet.
    { end: 126.4, start: 125.8, state: "new" },
  ],
  recordedPreset: "pip",
  rest: "still",
  scenes: [
    { end: 44.6, id: "scene-name", preset: "zoom", start: 39.8, zoom: 2 },
    { end: 61, id: "scene-templates", preset: "zoom", start: 55.4, zoom: 1.5 },
    { end: 74.2, id: "scene-create", preset: "zoom", start: 67.8, zoom: 2 },
    { end: 136.6, id: "scene-share", preset: "zoom", start: 124, zoom: 2 },
    { end: 155, id: "scene-recent", preset: "zoom", start: 150.4, zoom: 1.5 },
  ],
  sounds: [],
  speech: SPEECH,
});
