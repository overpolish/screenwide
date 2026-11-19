// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { describe, expect, it } from "vitest";

import { scriptText } from "../script-words";

import { cuesAfterEdit, wordAt } from "./cue-anchors";

const BEFORE = "Open the editor. Then press record.";
const at = (text: string, word: string) => text.indexOf(word);

/** Where a cue on `word` lands once `before` is edited into `after`. */
const landed = (before: string, after: string, word: string) => {
  const [cue] = cuesAfterEdit({ after, before, script: scriptText(after) }, [
    { at: at(before, word) },
  ]);
  return after.slice(cue.at).split(/\s/)[0];
};

describe("a cue as the script is edited", () => {
  it("stays on its word when text is typed before it", () => {
    expect(landed(BEFORE, `First, ${BEFORE}`, "press")).toBe("press");
  });

  it("stays put when the text after it changes", () => {
    const after = BEFORE.replace("record", "the button");
    expect(landed(BEFORE, after, "press")).toBe("press");
  });

  it("moves to the next word when its own is taken out", () => {
    expect(landed(BEFORE, BEFORE.replace("press ", ""), "press")).toBe(
      "record.",
    );
  });

  it("stays on its word when letters are joined to its start", () => {
    expect(landed(BEFORE, BEFORE.replace("press", "repress"), "press")).toBe(
      "repress",
    );
  });

  it("falls back to the last word when none follows", () => {
    expect(landed(BEFORE, "Open the", "press")).toBe("the");
  });
});

describe("the word at a place", () => {
  const script = scriptText(BEFORE);

  it("is the word the place falls in, or the next", () => {
    expect(wordAt(script, at(BEFORE, "editor") + 2)).toBe(2);
    expect(wordAt(script, at(BEFORE, "editor") - 1)).toBe(2);
  });

  it("is none in a script with no words", () => {
    expect(wordAt(scriptText(""), 0)).toBe(-1);
  });
});
