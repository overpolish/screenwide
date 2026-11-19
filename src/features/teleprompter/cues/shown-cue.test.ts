// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { describe, expect, it } from "vitest";

import { scriptText } from "../script-words";

import { LEAD_WORDS, shownCue } from "./shown-cue";

/** `count` words, each named by its place in the whole script. */
const paragraph = (from: number, count: number) =>
  Array.from({ length: count }, (_, index) => `w${String(from + index)}`).join(
    " ",
  );

// Three paragraphs of 20 words: words 0-19, 20-39 and 40-59.
const text = [paragraph(0, 20), paragraph(20, 20), paragraph(40, 20)].join(
  "\n\n",
);
const script = scriptText(text);
const cueAt = (word: number) => ({ at: script.starts[word], word });

describe("the cue shown while reading", () => {
  // Far enough into its paragraph that its lead stays within it.
  const first = cueAt(27);
  const second = cueAt(30);
  const last = cueAt(50);
  const cues = [second, first, last];

  it("is none before the first comes near", () => {
    expect(shownCue(cues, script, 27 - LEAD_WORDS - 1)).toBeNull();
  });

  it("shows a little before its word is read", () => {
    expect(shownCue(cues, script, 27 - LEAD_WORDS)).toBe(first);
  });

  it("keeps the cue being read while the next comes near", () => {
    expect(shownCue(cues, script, 30 - LEAD_WORDS)).toBe(first);
    expect(shownCue(cues, script, 29)).toBe(first);
  });

  it("gives way to the next once the next's word is reached", () => {
    expect(shownCue(cues, script, 30)).toBe(second);
  });

  it("goes once its paragraph has been read", () => {
    expect(shownCue(cues, script, 39)).toBe(second);
    expect(shownCue(cues, script, 40)).toBeNull();
  });

  it("stays to the end of the script in the last paragraph", () => {
    expect(shownCue(cues, script, 59)).toBe(last);
  });
});

describe("a cue near the start of its paragraph", () => {
  const before = cueAt(10);
  const after = cueAt(22);
  const cues = [before, after];

  it("leaves the paragraph before to its own cue to the end", () => {
    expect(shownCue(cues, script, 19)).toBe(before);
  });

  it("shows as its paragraph begins", () => {
    expect(shownCue(cues, script, 20)).toBe(after);
  });
});
