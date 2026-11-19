// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { describe, expect, it } from "vitest";

import { scriptText } from "./script-words";
import { followedPlace } from "./voice-follow";

const { spoken } = scriptText(
  [
    "# Welcome",
    "",
    "Hi, I'm Dom. Today we look at the **new** editor, and the way the",
    "editor keeps your place.",
    "",
    "- First, the timeline.",
    "- Then the islands, and the way the islands move.",
  ].join("\n"),
);

/** Reads `words` one at a time from the start, as they are heard. */
const readAlong = (words: string[], from = 0) => {
  let place = from;
  const heard: string[] = [];
  for (const word of words) {
    heard.push(word);
    place = followedPlace(spoken, heard, place);
  }
  return place;
};

describe("following the voice", () => {
  it("keeps the place on the word after the last one read", () => {
    expect(readAlong(["welcome", "hi", "I'm", "Dom", "today"])).toBe(5);
  });

  it("keeps up through a misheard word and a skipped one", () => {
    // "tom" for "Dom", "look" skipped.
    const place = readAlong([
      "welcome",
      "hi",
      "im",
      "tom",
      "today",
      "we",
      "at",
    ]);
    expect(spoken[place - 1]).toBe("at");
  });

  it("follows a word still being said", () => {
    const place = readAlong(["welcome", "hi", "im", "dom", "tod"]);
    expect(place).toBe(5);
  });

  it("is not pulled ahead to the same word later on", () => {
    // "the" and "editor" come up again further on.
    const place = readAlong(
      ["today", "we", "look", "at", "the", "new", "editor"],
      4,
    );
    expect(spoken[place - 1]).toBe("editor");
    expect(place).toBeLessThan(spoken.indexOf("keeps"));
  });

  it("goes back only for words clearly read again", () => {
    const at = spoken.indexOf("keeps") + 1;
    expect(followedPlace(spoken, ["look"], at)).toBe(at);
    expect(followedPlace(spoken, ["today", "we", "look"], at)).toBe(
      spoken.indexOf("look") + 1,
    );
  });
});
