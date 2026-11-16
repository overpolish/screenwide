// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  assignRun,
  editTurn,
  parseNotes,
  renameSpeaker,
  runsOf,
  speakerName,
  speakerShares,
  speakersOf,
  writeNotes,
} from "./notes-document";

// As the app writes them.
const NOTES = [
  "# Notes 2026-05-04 at 10.30.00",
  "",
  "**Me:** Can Speaker 1 take the docs?",
  "",
  "**Speaker 1:** Yes, by Friday.",
  "",
  "**Speaker 1:** I'll send it over.",
  "",
  "**Speaker 2:** Thanks.",
  "",
].join("\n");

describe("notes", () => {
  it("are written back exactly as the app wrote them", () => {
    expect(writeNotes(parseNotes(NOTES))).toBe(NOTES);
  });

  it("keep a paragraph typed by hand as no one's", () => {
    const notes = parseNotes(`${NOTES}\nAction items for next week.\n`);
    expect(notes.turns[notes.turns.length - 1]).toEqual({
      speaker: null,
      text: "Action items for next week.",
    });
    expect(writeNotes(notes)).toContain("\n\nAction items for next week.\n");
  });

  it("keep a turn one paragraph when a blank line is typed into it", () => {
    const notes = editTurn(parseNotes(NOTES), 1, "Yes.\n\n\nBy Friday.");
    expect(parseNotes(writeNotes(notes)).turns[1]).toEqual({
      speaker: "Speaker 1",
      text: "Yes.\nBy Friday.",
    });
  });

  it("show one speaker's turns in a row under their name once", () => {
    expect(runsOf(parseNotes(NOTES))).toEqual([
      { end: 1, speaker: "Me", start: 0 },
      { end: 3, speaker: "Speaker 1", start: 1 },
      { end: 4, speaker: "Speaker 2", start: 3 },
    ]);
  });

  it("rename a speaker in every turn of theirs and nowhere else", () => {
    const renamed = renameSpeaker(parseNotes(NOTES), "Speaker 1", "Priya");
    expect(speakersOf(renamed)).toEqual(["Me", "Priya", "Speaker 2"]);
    expect(writeNotes(renamed)).toContain(
      "**Me:** Can Speaker 1 take the docs?",
    );
  });

  it("give one run to another speaker and leave the rest", () => {
    const notes = parseNotes(NOTES);
    const [, speakerOne] = runsOf(notes);
    const given = assignRun(notes, speakerOne, "Speaker 2");
    expect(given.turns.map(({ speaker }) => speaker)).toEqual([
      "Me",
      "Speaker 2",
      "Speaker 2",
      "Speaker 2",
    ]);
  });

  it("take a name that cannot end a speaker's label early", () => {
    expect(speakerName("  **Dr: Lee** ")).toBe("Dr Lee");
  });

  it("share the words said among the speakers", () => {
    expect(speakerShares(parseNotes(NOTES))).toEqual([
      { name: "Me", share: 6 / 14 },
      { name: "Speaker 1", share: 7 / 14 },
      { name: "Speaker 2", share: 1 / 14 },
    ]);
  });
});
