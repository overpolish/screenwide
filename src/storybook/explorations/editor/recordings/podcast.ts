// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { recordingOf, type SpokenLine } from "../recording";

// Three minutes of one person talking to the camera into one microphone,
// recorded with the camera only. Nothing is done on a screen, so the voice
// carries it: each line is a beat, and the silences between are the
// springs. Against a talking head's own level the picture is at rest; it
// moves where they reach for their notes. Read from a real 10-minute talk
// (`screenwide-activity`, ffmpeg's silencedetect): a quarter of it silent,
// thirteen silences over three seconds, the longest fifteen.

const LINES: [number, number, string][] = [
  [0.8, 4.6, "So today I want to talk about premature optimisation."],
  [
    5.1,
    9.8,
    "Which is a fancy way of saying, fixing problems you don't have yet.",
  ],
  [10.4, 13.2, "And I've done this. A lot."],
  [
    14.9,
    20.6,
    "The last app I built, I spent a week on a cache before it had a single user.",
  ],
  [21.2, 23.0, "Um."],
  [23.6, 28.4, "And the cache was fine. It was a good cache."],
  [29.1, 31.0, "Nobody needed it."],
  [36.8, 42.5, "Sorry, I had a note about this somewhere. Right."],
  [
    43.0,
    49.6,
    "The rule I use now is simple: make it work, make it right, then make it fast.",
  ],
  [
    50.3,
    55.8,
    "In that order, and only go to the next step when the last one hurts.",
  ],
  [56.6, 61.2, "Measure first. Your guess about what's slow is usually wrong."],
  [62.0, 65.1, "Mine certainly is."],
  [
    66.4,
    72.9,
    "When I finally profiled that app, the slow part was a font loading.",
  ],
  [73.4, 76.3, "Not the database. Not the cache."],
  [77.1, 78.4, "A font."],
  [
    87.9,
    93.6,
    "Okay, so what does keeping it simple actually look like day to day?",
  ],
  [
    94.2,
    100.1,
    "For me it's writing the boring version first, the one I'd be embarrassed by.",
  ],
  [
    100.8,
    106.0,
    "Then living with it for a bit, and seeing where it actually breaks.",
  ],
  [106.7, 111.9, "Most of the time it doesn't. It just works, and I move on."],
  [113.2, 115.4, "Um, let me think."],
  [
    118.6,
    124.8,
    "The times it does break, you know exactly where, because you watched it.",
  ],
  [125.5, 131.7, "That's worth more than any amount of planning up front."],
  [132.4, 137.0, "So, if you take one thing from this:"],
  [
    137.6,
    142.3,
    "build the dumb thing first. Then measure. Then fix what hurts.",
  ],
  [148.6, 152.0, "That's it for this one."],
  [
    152.6,
    156.9,
    "If you've got a premature optimisation story, I'd love to hear it.",
  ],
  [157.5, 159.8, "See you next time."],
];

const SPEECH: SpokenLine[] = LINES.map(([start, end, text]) => ({
  end,
  start,
  text,
}));

export const PODCAST = recordingOf({
  actions: [],
  duration: 165,
  look: "camera",
  markTimes: {},
  // Flagged while talking: the line worth a short.
  moments: [140.2],
  picture: [
    // Reaching for the notes.
    { end: 36.4, start: 32.1, state: "moving" },
  ],
  recordedPreset: "camera",
  rest: "still",
  scenes: [],
  sounds: [],
  speech: SPEECH,
});
