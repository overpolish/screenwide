// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { describe, expect, it } from "vitest";

import { placeAfterEdit, scriptText } from "./script-words";

const SCRIPT = [
  "# Welcome",
  "",
  "Hi, I'm Dom. Today we look at the **new** editor.",
  "",
  "- First, the timeline.",
  "1. Then the islands.",
].join("\n");

const edited = (markdown: string) => ({ markdown, text: scriptText(markdown) });

describe("the script as it is shown and read", () => {
  const text = scriptText(SCRIPT);

  it("shows every character as written, so the field over it lines up", () => {
    expect(text.pieces.map((piece) => piece.text).join("")).toBe(SCRIPT);
  });

  it("reads the words and leaves the marks that only style them", () => {
    expect(text.spoken.slice(0, 5)).toEqual([
      "welcome",
      "hi",
      "im",
      "dom",
      "today",
    ]);
    expect(text.spoken).toContain("new");
    expect(text.spoken).not.toContain("1");
    const marks = text.pieces
      .filter((piece) => piece.kind === "mark")
      .map((piece) => piece.text);
    expect(marks).toEqual(["#", "**", "**", "-", "1."]);
  });

  it("knows where each word starts in the text", () => {
    const index = text.spoken.indexOf("new");
    expect(SCRIPT.slice(text.starts[index], text.starts[index] + 3)).toBe(
      "new",
    );
  });
});

describe("the place after an edit", () => {
  const before = edited(SCRIPT);
  const place = before.text.spoken.indexOf("editor");

  it("stays on its word when words are added ahead of it", () => {
    const after = edited(SCRIPT.replace("Hi,", "Hello there, hi,"));
    expect(after.text.spoken[placeAfterEdit(before, after, place)]).toBe(
      "editor",
    );
  });

  it("stays on its word when words after it change", () => {
    const after = edited(SCRIPT.replace("timeline", "track list"));
    expect(placeAfterEdit(before, after, place)).toBe(place);
  });

  it("moves to the next word when its own is taken out", () => {
    const after = edited(SCRIPT.replace(" editor.", "."));
    expect(after.text.spoken[placeAfterEdit(before, after, place)]).toBe(
      "first",
    );
  });
});
