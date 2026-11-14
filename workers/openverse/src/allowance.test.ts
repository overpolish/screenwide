// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  ADDRESS_PER_DAY,
  PERSON_PER_DAY,
  personOf,
  take,
  type Tally,
} from "./allowance";

const tallyOf = (): Tally => {
  const kept = new Map<string, unknown>();
  return {
    delete: (key) => kept.delete(key),
    get: (key) => kept.get(key),
    list: () => kept.entries(),
    put: (key, value) => {
      kept.set(key, value);
    },
  };
};

const NOON = Date.UTC(2026, 10, 14, 12);
const install = (n: number) => personOf(n.toString(16).padStart(32, "0"));

describe("a day's allowance", () => {
  it("stops a person at their share while others at the address go on", () => {
    const tally = tallyOf();
    for (let i = 0; i < PERSON_PER_DAY; i += 1)
      expect(take(tally, install(1), NOON)).toBe(true);
    expect(take(tally, install(1), NOON)).toBe(false);
    expect(take(tally, install(2), NOON)).toBe(true);
  });

  it("stops an address however many installations it makes up", () => {
    const tally = tallyOf();
    for (let i = 0; i < ADDRESS_PER_DAY; i += 1)
      expect(take(tally, install(i), NOON)).toBe(true);
    expect(take(tally, install(ADDRESS_PER_DAY), NOON)).toBe(false);
  });

  it("starts again the next day", () => {
    const tally = tallyOf();
    for (let i = 0; i < PERSON_PER_DAY; i += 1) take(tally, install(1), NOON);
    expect(take(tally, install(1), NOON + 12 * 3_600_000)).toBe(true);
  });

  it("counts everyone sending no ID, or a made-up one, as one person", () => {
    expect(personOf(null)).toBe(personOf("not an id"));
    expect(install(1)).not.toBe(personOf(null));
  });
});
