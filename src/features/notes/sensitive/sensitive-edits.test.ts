// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  groupByKind,
  markNotPrivate,
  REDACTED,
  redactInTurn,
  redactThroughout,
  withoutNotPrivate,
} from "./sensitive-edits";

import type { NotesDocument } from "../document/notes-document";

const notes: NotesDocument = {
  title: "Call",
  turns: [
    {
      speaker: "Me",
      text: "Ask Priya Shah, priya@northwind.com, or Priya's team.",
    },
    { speaker: "Speaker 2", text: "priya will sign." },
  ],
};

describe("redacting what may be private", () => {
  it("redacts each find in one paragraph and leaves the rest", () => {
    const redacted = redactInTurn(notes, 0, [
      { end: 14, start: 4 },
      { end: 35, start: 16 },
    ]);
    expect(redacted.turns[0].text).toBe(
      `Ask ${REDACTED}, ${REDACTED}, or Priya's team.`,
    );
    expect(redacted.turns[1]).toBe(notes.turns[1]);
  });

  it("redacts words throughout the notes in any case", () => {
    const redacted = redactThroughout(notes, "Priya");
    expect(redacted.turns.map(({ text }) => text)).toEqual([
      `Ask ${REDACTED} Shah, ${REDACTED}@northwind.com, or ${REDACTED}'s team.`,
      `${REDACTED} will sign.`,
    ]);
  });

  it("groups finds by kind in the order each kind first appears", () => {
    const groups = groupByKind([
      { end: 2, kind: "phone", start: 0 },
      { end: 6, kind: "name", start: 4 },
      { end: 9, kind: "phone", start: 8 },
    ]);
    expect(groups.map(({ finds, kind }) => [kind, finds.length])).toEqual([
      ["phone", 2],
      ["name", 1],
    ]);
  });

  it("stops pointing out words marked not private, in any case", () => {
    const marked = markNotPrivate(notes, ["Priya Shah"]);
    const lookup = withoutNotPrivate(
      () => [
        { end: 14, kind: "name", start: 4 },
        { end: 35, kind: "email", start: 16 },
      ],
      markNotPrivate(marked, ["PRIYA SHAH"]).notPrivate,
    );
    expect(marked.notPrivate).toEqual(["priya shah"]);
    expect(lookup(0, notes.turns[0].text)).toEqual([
      { end: 35, kind: "email", start: 16 },
    ]);
  });

  it("does not point out words already redacted", () => {
    const redacted = redactInTurn(notes, 0, [{ end: 14, start: 4 }]);
    const text = redacted.turns[0].text;
    const lookup = withoutNotPrivate(() => [
      { end: 4 + REDACTED.length, kind: "company", start: 4 },
    ]);
    expect(lookup(0, text)).toEqual([]);
  });
});
