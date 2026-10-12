// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { nextCutRun, nextUnsureWord } from "./transcript-cuts";

const removedFlags = (...flags: boolean[]) =>
  flags.map((removed) => ({ removed }));

describe("nextCutRun", () => {
  it("finds the next run of struck units after `from`", () => {
    const units = removedFlags(false, false, true, true, false, true);
    expect(nextCutRun(units, 0, 1)).toEqual({ first: 2, last: 3 });
    expect(nextCutRun(units, 3, 1)).toEqual({ first: 5, last: 5 });
  });

  it("skips the run `from` is already inside, rather than staying on it", () => {
    const units = removedFlags(true, true, false, true);
    expect(nextCutRun(units, 0, 1)).toEqual({ first: 3, last: 3 });
  });

  it("finds the previous run going backward", () => {
    const units = removedFlags(true, false, false, true, true, false);
    expect(nextCutRun(units, 5, -1)).toEqual({ first: 3, last: 4 });
    expect(nextCutRun(units, 3, -1)).toEqual({ first: 0, last: 0 });
  });

  it("is null once there is no other cut in that direction", () => {
    const units = removedFlags(false, true, false);
    expect(nextCutRun(units, 1, 1)).toBeNull();
    expect(nextCutRun(units, 1, -1)).toBeNull();
  });
});

const unsureFlags = (
  ...flags: { removed?: boolean; unsure?: boolean; word?: boolean }[]
) =>
  flags.map(({ removed = false, unsure = false, word = true }) => ({
    kind: word ? ("word" as const) : ("pause" as const),
    removed,
    unsure,
  }));

describe("nextUnsureWord", () => {
  it("finds the next unsure word kept rather than struck", () => {
    const units = unsureFlags(
      {},
      { unsure: true },
      {},
      { removed: true, unsure: true },
      { unsure: true },
    );
    expect(nextUnsureWord(units, 0, 1)).toBe(1);
    expect(nextUnsureWord(units, 1, 1)).toBe(4);
  });

  it("skips a pause even where it would otherwise be unsure", () => {
    const units = unsureFlags({ unsure: true, word: false }, { unsure: true });
    expect(nextUnsureWord(units, -1, 1)).toBe(1);
  });

  it("finds the previous unsure word going backward", () => {
    const units = unsureFlags({ unsure: true }, {}, { unsure: true });
    expect(nextUnsureWord(units, 2, -1)).toBe(0);
  });

  it("is -1 once there is no other unsure word in that direction", () => {
    const units = unsureFlags({ unsure: true }, {});
    expect(nextUnsureWord(units, 0, 1)).toBe(-1);
  });
});
