// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// How many questions each person may send on to Openverse in a day, so no
// one uses up what the app's registration allows everyone (10,000 a day).
// A person is an installation of the app at an address: the app sends a
// random ID made for its installation, which tells apart people sharing an
// address, as in an office. An ID costs nothing to make up, so the address
// as a whole has a ceiling too. Only questions sent on count: an answer
// already in the cache costs Openverse nothing. Counts last the day (UTC)
// and are then forgotten.

/** Questions one installation may send on in a day. */
export const PERSON_PER_DAY = 300;

/** Questions one address may send on in a day, whoever sends them. */
export const ADDRESS_PER_DAY = 1_500;

/** The header the app names its installation in. */
export const INSTALL_HEADER = "X-Screenwide-Install";

/** What an installation's ID looks like: 128 random bits in hex. */
const INSTALL = /^[0-9a-f]{32}$/;

/** Who is asking at an address, by the ID the app sent; those sending none,
 * as an older app does, count as one. */
export const personOf = (header: string | null) =>
  header !== null && INSTALL.test(header) ? `install:${header}` : "unnamed";

/** Where an address's counts are kept: each key a count, beside the day
 * they are for. */
export type Tally = {
  delete(key: string): boolean;
  get(key: string): unknown;
  list(): Iterable<[string, unknown]>;
  put(key: string, value: unknown): void;
};

const countOf = (value: unknown) => (typeof value === "number" ? value : 0);

const DAY = "day";
const ADDRESS = "address";

/** The day `now` falls in, UTC, as the counts are kept. */
const dayOf = (now: number) => new Date(now).toISOString().slice(0, 10);

/** When the day `now` falls in ends, UTC, in ms since the epoch. */
export const endOfDay = (now: number) => {
  const end = new Date(now);
  end.setUTCHours(24, 0, 0, 0);
  return end.getTime();
};

/** Counts one question from `person` at `now`, where they and their address
 * have any left today, and says whether it may be sent on. */
export function take(tally: Tally, person: string, now: number) {
  const today = dayOf(now);
  if (tally.get(DAY) !== today) {
    for (const [key] of [...tally.list()]) tally.delete(key);
    tally.put(DAY, today);
  }
  const address = countOf(tally.get(ADDRESS));
  const own = countOf(tally.get(person));
  if (address >= ADDRESS_PER_DAY || own >= PERSON_PER_DAY) return false;
  tally.put(ADDRESS, address + 1);
  tally.put(person, own + 1);
  return true;
}
