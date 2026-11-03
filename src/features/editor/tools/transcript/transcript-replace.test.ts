// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { replacementCorrections } from "./transcript-replace";
import { findInTranscript } from "./transcript-search";
import { TranscriptUnit } from "./transcript-units";

const unit = (text: string, startMs: number): TranscriptUnit => ({
  corrected: false,
  endMs: startMs + 400,
  heard: text,
  kind: "word",
  removed: false,
  startMs,
  text,
  unsure: false,
});

describe("replacementCorrections", () => {
  it("replaces only the matched part of a single unit, keeping the rest", () => {
    const units = [unit("Worldwide", 0)];
    const matches = findInTranscript(units, "wide");
    expect(replacementCorrections(units, matches, "class")).toEqual([
      { endMs: 400, startMs: 0, text: "Worldclass" },
    ]);
  });

  it("joins a match across units into one phrase correction", () => {
    const units = [unit("the", 0), unit("Café,", 500), unit("then", 1_000)];
    const matches = findInTranscript(units, "e caf");
    expect(replacementCorrections(units, matches, "a bar")).toEqual([
      { endMs: 900, startMs: 0, text: "tha baré," },
    ]);
  });

  it("skips a match that already reads as the replacement", () => {
    const units = [unit("Open", 0), unit("the", 500), unit("door.", 1_000)];
    const matches = findInTranscript(units, "open");
    expect(replacementCorrections(units, matches, "Open")).toEqual([]);
  });

  it("builds one correction per match, each over its own units", () => {
    const units = [
      unit("open", 0),
      unit("it", 500),
      unit("then", 1_000),
      unit("open", 1_500),
      unit("it", 2_000),
    ];
    const matches = findInTranscript(units, "open it");
    expect(replacementCorrections(units, matches, "close it")).toEqual([
      { endMs: 900, startMs: 0, text: "close it" },
      { endMs: 2_400, startMs: 1_500, text: "close it" },
    ]);
  });
});
