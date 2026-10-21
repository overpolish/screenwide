// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { matchSettings } from "./match-settings";

import type { SettingsSearchEntry } from "./settings-search-index";

const entry = (
  title: string,
  extra: Partial<SettingsSearchEntry> = {},
): SettingsSearchEntry => ({
  id: title,
  location: "Tools › Glide",
  section: "tools",
  target: "row",
  title,
  ...extra,
});

const entries = [
  entry("Space around windows", { keywords: "padding margin gap" }),
  entry("Move pointer with window"),
  entry("Show or hide recording controls", {
    keywords: "hotkey keyboard shortcut",
    location: "Shortcuts › Recording",
  }),
  entry("Recording countdown", {
    keywords: "timer delay",
    location: "General › Recording",
  }),
  entry("Délai d’enregistrement"),
];

const titles = (query: string) =>
  matchSettings(entries, query).map(({ title }) => title);

describe("settings search", () => {
  it("matches the starts of words, in any order", () => {
    expect(titles("mov poin")).toEqual(["Move pointer with window"]);
    expect(titles("pointer move")).toEqual(["Move pointer with window"]);
  });

  it("matches abbreviations", () => {
    expect(titles("mov ptr")).toEqual(["Move pointer with window"]);
  });

  it("forgives a typo in a longer word", () => {
    expect(titles("shorcut")).toEqual(["Show or hide recording controls"]);
    expect(titles("countdonw")).toEqual(["Recording countdown"]);
  });

  it("does not let a short word match anything a typo away", () => {
    expect(titles("gap")).toEqual(["Space around windows"]);
  });

  it("finds a setting by the other words it is known by", () => {
    expect(titles("padding")).toEqual(["Space around windows"]);
    expect(titles("timer")).toEqual(["Recording countdown"]);
  });

  it("ignores accents and case", () => {
    expect(titles("DELAI")[0]).toBe("Délai d’enregistrement");
  });

  it("needs every word typed to match", () => {
    expect(titles("recording padding")).toEqual([]);
  });

  it("ranks a match in the title above one in the other words", () => {
    const both = [entry("Countdown", { keywords: "timer" }), entry("Timer")];
    expect(matchSettings(both, "timer").map(({ title }) => title)).toEqual([
      "Timer",
      "Countdown",
    ]);
  });

  it("ranks a whole word above the start of one", () => {
    expect(titles("window")).toEqual([
      "Move pointer with window",
      "Space around windows",
    ]);
  });

  it("finds nothing for an empty query", () => {
    expect(titles("  ")).toEqual([]);
  });
});
