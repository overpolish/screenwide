// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { correctedLines } from "./corrected-words";

const word = (start: number, end: number, text: string) => ({
  end,
  start,
  text,
});

const heard = [
  word(1, 1.4, "Hello?"),
  word(3, 3.3, "see"),
  word(3.4, 3.6, "you"),
  word(3.7, 4, "later."),
];

describe("correctedLines", () => {
  it("reads a corrected word under its correction, where it was heard", () => {
    const lines = correctedLines(heard, [
      { endMs: 1400, startMs: 1000, text: "Howdy?" },
    ]);
    expect(lines.map((line) => line.text)).toEqual([
      "Howdy?",
      "see you later.",
    ]);
    expect(lines[0]).toMatchObject({
      end: 1.4,
      start: 1,
      words: [{ at: 1, word: "Howdy?" }],
    });
  });

  it("joins the words one correction covers, and leaves out a hidden one", () => {
    const lines = correctedLines(heard, [
      { endMs: 1400, startMs: 1000, text: "" },
      { endMs: 3600, startMs: 3000, text: "seeya" },
    ]);
    expect(lines).toHaveLength(1);
    expect(lines[0]).toMatchObject({
      end: 4,
      start: 3,
      text: "seeya later.",
      words: [
        { at: 3, word: "seeya" },
        { at: 3.7, word: "later." },
      ],
    });
  });

  it("keeps the lines where they were heard as a correction ends a sentence", () => {
    const lines = correctedLines(heard, [
      { endMs: 3600, startMs: 3400, text: "you." },
    ]);
    expect(lines.map((line) => line.text)).toEqual([
      "Hello?",
      "see you. later.",
    ]);
  });
});
