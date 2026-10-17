// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { isChosenWord, textWordLines } from "./text-words";

describe("textWordLines", () => {
  it("names each word by its bytes in the text's UTF-8, as Rust does", () => {
    // "é" is two bytes, "😀" four, and a no-break space parts words too.
    const lines = textWordLines("café 😀ok\n\n  a\u00a0b");
    expect(lines).toEqual([
      [
        { span: { end: 5, start: 0 }, text: "café" },
        { span: { end: 12, start: 6 }, text: "😀ok" },
      ],
      [
        { span: { end: 17, start: 16 }, text: "a" },
        { span: { end: 20, start: 19 }, text: "b" },
      ],
    ]);
  });

  it("reads a word as chosen where a stored range reaches into it", () => {
    const [[first, second]] = textWordLines("big cat");
    const emphasis = [{ end: 6, start: 5 }];
    expect(isChosenWord(emphasis, first)).toBe(false);
    expect(isChosenWord(emphasis, second)).toBe(true);
  });
});
