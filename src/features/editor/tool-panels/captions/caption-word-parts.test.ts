// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { captionWordParts, withCaptionPunctuation } from "./caption-word-parts";

describe("captionWordParts", () => {
  it("keeps marks inside a word with it", () => {
    expect(captionWordParts("(don't),")).toEqual({
      after: "),",
      before: "(",
      word: "don't",
    });
  });

  it("leaves a lone mark editable as the word", () => {
    expect(captionWordParts("—")).toEqual({ after: "", before: "", word: "—" });
  });
});

describe("withCaptionPunctuation", () => {
  const parts = captionWordParts("captions,");

  it("keeps the punctuation when only the word is typed", () => {
    expect(withCaptionPunctuation(parts, "subtitles")).toBe("subtitles,");
  });

  it("takes the typed punctuation on the side it was typed", () => {
    expect(withCaptionPunctuation(parts, "subtitles.")).toBe("subtitles.");
    expect(withCaptionPunctuation(parts, "¿subtitles")).toBe("¿subtitles,");
  });
});
