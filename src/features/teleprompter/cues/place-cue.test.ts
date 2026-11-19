// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { describe, expect, it } from "vitest";

import { scriptText } from "../script-words";

import { placeCue } from "./place-cue";

const text = "one two three four five six seven eight";
const script = scriptText(text);
// Laid out two words to a line: one two / three four / five six / seven eight.
const layout = { lineOf: (index: number) => Math.floor(index / 2), script };
const cue = (id: string, index: number) => ({ at: script.starts[index], id });
/** Each cue's word, by id. */
const words = (cues: readonly { at: number; id: string }[]) =>
  Object.fromEntries(
    cues.map(({ at, id }) => [id, text.slice(at).split(" ")[0]]),
  );

describe("moving a cue", () => {
  it("takes it to the word it is dropped on", () => {
    expect(words(placeCue([cue("a", 0)], cue("a", 3), { script }))).toEqual({
      a: "four",
    });
  });

  it("moves a cue on that line down to the next line with none", () => {
    const cues = [cue("a", 0), cue("b", 2)];
    expect(words(placeCue(cues, cue("a", 3), layout))).toEqual({
      a: "four",
      b: "five",
    });
  });

  it("passes over lines that hold a cue, leaving it there", () => {
    const cues = [cue("a", 0), cue("b", 2), cue("c", 4)];
    expect(words(placeCue(cues, cue("a", 3), layout))).toEqual({
      a: "four",
      b: "seven",
      c: "five",
    });
  });

  it("takes the line the moved cue left", () => {
    const cues = [cue("a", 0), cue("b", 2)];
    expect(words(placeCue(cues, cue("b", 1), layout))).toEqual({
      a: "three",
      b: "two",
    });
  });

  it("leaves a cue with no free line after it where it was", () => {
    const cues = [cue("a", 0), cue("b", 6)];
    expect(words(placeCue(cues, cue("a", 7), layout))).toEqual({
      a: "eight",
      b: "seven",
    });
  });
});
