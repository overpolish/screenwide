// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { describe, expect, it } from "vitest";

import { placeAfterEdit, scriptText } from "./script-words";

const SCRIPT = [
  "Welcome",
  "",
  "Hi, I'm Dom. Today we look at the new editor.",
  "",
  "First - the timeline. Then the islands.",
].join("\n");

const written = (text: string) => ({ script: scriptText(text), text });

describe("the script as it is shown and read", () => {
  const { pieces, spoken, starts } = scriptText(SCRIPT);

  it("shows every character as written, so the field over it lines up", () => {
    expect(pieces.map((piece) => piece.text).join("")).toBe(SCRIPT);
  });

  it("reads the words, and not a mark standing on its own", () => {
    expect(spoken.slice(0, 5)).toEqual(["welcome", "hi", "im", "dom", "today"]);
    // The dash between "First" and "the" is shown but not read.
    expect(spoken.slice(-6)).toEqual([
      "first",
      "the",
      "timeline",
      "then",
      "the",
      "islands",
    ]);
  });

  it("knows where each word starts in the text", () => {
    const index = spoken.indexOf("new");
    expect(SCRIPT.slice(starts[index], starts[index] + 3)).toBe("new");
  });
});

describe("the place after an edit", () => {
  const before = written(SCRIPT);
  const place = before.script.spoken.indexOf("editor");

  it("stays on its word when words are added ahead of it", () => {
    const after = written(SCRIPT.replace("Hi,", "Hello there, hi,"));
    expect(after.script.spoken[placeAfterEdit(before, after, place)]).toBe(
      "editor",
    );
  });

  it("stays on its word when words after it change", () => {
    const after = written(SCRIPT.replace("timeline", "track list"));
    expect(placeAfterEdit(before, after, place)).toBe(place);
  });

  it("moves to the next word when its own is taken out", () => {
    const after = written(SCRIPT.replace(" editor.", "."));
    expect(after.script.spoken[placeAfterEdit(before, after, place)]).toBe(
      "first",
    );
  });
});
