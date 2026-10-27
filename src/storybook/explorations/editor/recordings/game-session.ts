// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { recordingOf } from "../recording";

// Three minutes of a first-person match, as the readings would find it. The
// keys and mouse never stop, so what was done says nothing and is left out.
// The picture moves all the time, so it is at rest moving, and what stands
// out is where it stops: the load, the wait after dying, the scoreboard
// between rounds, the result. Fights are heard as loud moments; two were
// flagged with the moments key while playing; a few lines go out over voice
// chat. Read from real play (`screenwide-activity`, Red Eclipse): death
// screens, scoreboards and loads read still against play, and each cut to a
// new screen reads new.

export const GAME_SESSION = recordingOf({
  actions: [],
  duration: 180,
  look: "game",
  markTimes: {},
  moments: [70.5, 163.8],
  picture: [
    { end: 6, label: "Loading Harbour", start: 0, state: "still" },
    { end: 6.6, start: 6, state: "new" },
    { end: 53, label: "Respawning in 5", start: 48, state: "still" },
    { end: 101, label: "Round 1 · 4 – 2", start: 92, state: "still" },
    { end: 101.6, start: 101, state: "new" },
    { end: 145, label: "Respawning in 5", start: 140, state: "still" },
    { end: 180, label: "Victory", start: 171, state: "still" },
    { end: 171.6, start: 171, state: "new" },
  ],
  recordedPreset: "screen",
  rest: "moving",
  scenes: [],
  sounds: [14.2, 31.8, 44.5, 47.6, 66.3, 118.4, 121, 139.2, 158.6],
  speech: [
    { end: 17.2, start: 15.6, text: "Got him!" },
    { end: 71.2, start: 69, text: "Two! Did you see that?" },
    { end: 122, start: 120.4, text: "Behind you, behind you." },
    { end: 166.2, start: 164, text: "That's the round." },
  ],
});
