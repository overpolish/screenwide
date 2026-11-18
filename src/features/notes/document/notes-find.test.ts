// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { parseNotes, writeNotes } from "./notes-document";
import { findInNotes, replaceMatches } from "./notes-find";

const NOTES = parseNotes(
  [
    "# Call",
    "",
    "**Me:** Did Priya send it?",
    "",
    "**Priya:** priya sent it. Ask Priya or PRIYA's team.",
    "",
  ].join("\n"),
);

describe("finding in notes", () => {
  it("finds every match in what was said, whatever its case, and no name", () => {
    expect(findInNotes(NOTES, "priya")).toEqual([
      { from: 4, to: 9, turn: 0 },
      { from: 0, to: 5, turn: 1 },
      { from: 19, to: 24, turn: 1 },
      { from: 28, to: 33, turn: 1 },
    ]);
  });

  it("replaces every match and keeps the speaker's name", () => {
    const replaced = replaceMatches(
      NOTES,
      findInNotes(NOTES, "priya"),
      "Speaker A",
    );
    expect(writeNotes(replaced)).toBe(
      [
        "# Call",
        "",
        "**Me:** Did Speaker A send it?",
        "",
        "**Priya:** Speaker A sent it. Ask Speaker A or Speaker A's team.",
        "",
      ].join("\n"),
    );
  });

  it("replaces one match and leaves the others", () => {
    const [, second] = findInNotes(NOTES, "priya");
    const replaced = replaceMatches(NOTES, [second], "She");
    expect(replaced.turns[1].text).toBe(
      "She sent it. Ask Priya or PRIYA's team.",
    );
    expect(findInNotes(replaced, "priya")).toHaveLength(3);
  });
});
