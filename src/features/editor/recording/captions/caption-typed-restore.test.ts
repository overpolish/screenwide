// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { captionTypedRestore } from "./caption-typed-restore";

describe("captionTypedRestore", () => {
  it("puts back only the word pressed where each stands for one heard", () => {
    // "the quick fox" typed as "a slow fox" from byte 10 of the caption.
    const run = { start: 10, text: "a slow fox" };
    expect(captionTypedRestore(run, 12, ["the", "quick", "fox"])).toStrictEqual(
      { end: 16, start: 12, text: "quick" },
    );
    expect(captionTypedRestore(run, 10, ["the", "quick", "fox"])).toStrictEqual(
      { end: 11, start: 10, text: "the" },
    );
  });

  it("offers nothing for a word that reads as it was heard", () => {
    expect(
      captionTypedRestore({ start: 0, text: "a big cat" }, 0, ["a", "cat"]),
    ).toBeNull();
  });

  it("takes out a word typed where nothing was heard", () => {
    expect(
      captionTypedRestore({ start: 0, text: "a big cat" }, 2, ["a", "cat"]),
    ).toStrictEqual({ end: 5, start: 2, text: "" });
  });

  it("puts back together words typed for a different number heard", () => {
    expect(
      captionTypedRestore({ start: 0, text: "so anyway than" }, 3, [
        "so",
        "any",
        "way",
        "then",
      ]),
    ).toStrictEqual({ end: 14, start: 3, text: "any way then" });
  });

  it("counts its bytes as the caption's text does", () => {
    // "é" is two bytes, so "cafe" after it starts at byte 4.
    expect(
      captionTypedRestore({ start: 0, text: "né cafe" }, 4, ["né", "café"]),
    ).toStrictEqual({ end: 8, start: 4, text: "café" });
  });
});
