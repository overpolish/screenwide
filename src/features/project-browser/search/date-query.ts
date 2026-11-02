// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fold, type Phrases } from "./date-names";
import {
  DAY_PARTS,
  resolveRange,
  type DateRange,
  type DayPart,
  type Facet,
  type Period,
} from "./date-range";

// When something was recorded, read from a search's words: a day ("today",
// "monday", "12 oct"), perhaps with a time of day ("3pm", "morning"); a
// month or year; a week or month ("last week"); or a recent span ("2 hours
// ago"). What is not part of the date is left for the word search.

export type Keyword =
  | "ago"
  | "am"
  | "hours"
  | "last"
  | "minutes"
  | "one"
  | "pm"
  | "today"
  | "yesterday"
  | DayPart
  | Period;

export type DateVocabulary = {
  /** The locale the names are folded in. */
  locale: string;
  /** Folded month names by month, 0 for January. */
  months: ReadonlyMap<string, number>;
  /** The day weeks start on, by `Date.getDay`. */
  weekStart: number;
  /** Folded weekday names by `Date.getDay`, 0 for Sunday. */
  weekdays: ReadonlyMap<string, number>;
  words: Readonly<Record<Keyword, Phrases>>;
};

export type DateQuery = DateRange & {
  /** The query's words that are not the date, as typed, for the word search. */
  words: string;
};

const PHRASES = [
  "today",
  "yesterday",
  "this-week",
  "last-week",
  "this-month",
  "last-month",
  "morning",
  "afternoon",
  "evening",
  "night",
] as const;

type Match = { facet: Facet; length: number };

/**
 * The date in `query`, read at `now` in `vocabulary`'s words, with the
 * range it covers and the words left over; null when it names no date, or
 * names one in two ways ("monday tuesday"), so the query is words alone.
 */
export function parseDateQuery(
  query: string,
  now: Date,
  vocabulary: DateVocabulary,
): DateQuery | null {
  const originals = query.trim().split(/\s+/u).filter(Boolean);
  // A word of digits and letters ("3pm", "30min", "12oct") is read as two.
  const tokens = originals.flatMap((original, from) => {
    const folded = fold(original, vocabulary.locale);
    const split = /^(\d+)(\p{L}+)$/u.exec(folded);
    return (split ? [split[1], split[2]] : [folded])
      .filter(Boolean)
      .map((text) => ({ from, text }));
  });
  const texts = tokens.map(({ text }) => text);
  const used = tokens.map(() => false);
  const facets: Facet[] = [];
  for (let index = 0; index < texts.length;) {
    const match = matchAt({ now, texts, vocabulary }, index);
    if (!match) {
      index += 1;
      continue;
    }
    facets.push(match.facet);
    used.fill(true, index, index + match.length);
    index += match.length;
  }
  if (facets.length === 0) return null;
  const words: string[] = [];
  for (const [from, original] of originals.entries()) {
    const own = tokens.flatMap((token, index) =>
      token.from === from ? [used[index]] : [],
    );
    // Part of a word read as a date and part not is no date at all.
    if (own.some(Boolean) && !own.every(Boolean)) return null;
    if (!own.some(Boolean)) words.push(original);
  }
  const resolved = resolveRange(facets, now, vocabulary.weekStart);
  return resolved && { ...resolved, words: words.join(" ") };
}

/** The longest phrase of `phrases` the words from `index` begin with, in
 * words; 0 for none. */
function phraseAt(phrases: Phrases, texts: readonly string[], index: number) {
  let longest = 0;
  for (const phrase of phrases)
    if (
      phrase.length > longest &&
      phrase.every((word, offset) => texts[index + offset] === word)
    )
      longest = phrase.length;
  return longest;
}

const numberAt = (texts: readonly string[], index: number, digits: RegExp) =>
  digits.test(texts[index] ?? "") ? Number(texts[index]) : null;

/** The query's folded words, read at `now`. */
type Scan = {
  now: Date;
  texts: readonly string[];
  vocabulary: DateVocabulary;
};

function matchAt(scan: Scan, index: number): Match | null {
  const { texts } = scan;
  const { weekdays } = scan.vocabulary;
  return (
    recentAt(scan, index) ??
    timeAt(scan, index) ??
    calendarAt(scan, index) ??
    phrasesAt(scan, index) ??
    (weekdays.has(texts[index])
      ? {
          facet: { kind: "weekday", weekday: weekdays.get(texts[index]) ?? 0 },
          length: 1,
        }
      : null)
  );
}

function phrasesAt(
  { texts, vocabulary: { words } }: Scan,
  index: number,
): Match | null {
  for (const key of PHRASES) {
    const length = phraseAt(words[key], texts, index);
    if (length === 0) continue;
    const facet: Facet =
      key === "today" || key === "yesterday"
        ? { days: key === "today" ? 0 : 1, kind: "relative" }
        : key in DAY_PARTS
          ? { kind: "part", part: key as DayPart }
          : { kind: "period", period: key as Period };
    return { facet, length };
  }
  return null;
}

// "2 hours ago", "an hour ago", "last hour", "past 30 minutes", and "vor 2
// Stunden" where a language puts its "ago" first. A number of hours alone
// ("2 hours") is not a time and is left as words.
function recentAt(
  { texts, vocabulary: { words } }: Scan,
  index: number,
): Match | null {
  let at = index;
  const agoBefore = phraseAt(words.ago, texts, at);
  at += agoBefore;
  const last = phraseAt(words.last, texts, at);
  at += last;
  let count = numberAt(texts, at, /^\d{1,3}$/u);
  if (count !== null) at += 1;
  else {
    const one = phraseAt(words.one, texts, at);
    at += one;
    if (one > 0 || last > 0 || agoBefore > 0) count = 1;
  }
  const hours = phraseAt(words.hours, texts, at);
  const minutes = hours > 0 ? 0 : phraseAt(words.minutes, texts, at);
  if (hours + minutes === 0) return null;
  at += hours + minutes;
  const agoAfter = agoBefore > 0 ? 0 : phraseAt(words.ago, texts, at);
  at += agoAfter;
  if (agoBefore + last + agoAfter === 0 || !count) return null;
  const unit = hours > 0 ? "hour" : "minute";
  return { facet: { count, kind: "recent", unit }, length: at - index };
}

// "3pm", "3 pm", "12 a.m.", "15:00", "9.30"; a number alone is no time.
function timeAt(
  { texts, vocabulary: { words } }: Scan,
  index: number,
): Match | null {
  const time = /^(?<hour>\d{1,2})(?:[:.](?<minutes>\d{2}))?$/u.exec(
    texts[index] ?? "",
  );
  if (!time?.groups) return null;
  const hour = Number(time.groups.hour);
  // Absent when only an hour is written.
  const minutes = time.groups.minutes as string | undefined;
  const am = phraseAt(words.am, texts, index + 1);
  const pm = am > 0 ? 0 : phraseAt(words.pm, texts, index + 1);
  if (am + pm > 0) {
    if (hour < 1 || hour > 12) return null;
    const hour24 = (hour % 12) + (pm > 0 ? 12 : 0);
    return { facet: { hour: hour24, kind: "hour" }, length: 1 + am + pm };
  }
  if (minutes === undefined || hour > 23 || Number(minutes) > 59) return null;
  return { facet: { hour, kind: "hour" }, length: 1 };
}

// "12 oct", "oct 12", either with a year after; "october", "october 2025";
// "2025". A number before "am" or "pm" is an hour, not a day.
function calendarAt(
  { now, texts, vocabulary: { months, words } }: Scan,
  index: number,
): Match | null {
  const yearAt = (at: number) => {
    const year = numberAt(texts, at, /^\d{4}$/u);
    return year !== null && year >= 1970 && year <= now.getFullYear()
      ? year
      : null;
  };
  const dayAt = (at: number) => {
    const day = numberAt(texts, at, /^\d{1,2}$/u);
    const isHour =
      phraseAt(words.am, texts, at + 1) + phraseAt(words.pm, texts, at + 1) > 0;
    return day !== null && day >= 1 && day <= 31 && !isHour ? day : null;
  };
  const dated = (day: number, month: number, at: number): Match => {
    const year = yearAt(at);
    return {
      facet: { day, kind: "date", month, year },
      length: at - index + (year === null ? 0 : 1),
    };
  };
  const first = dayAt(index);
  const monthAfter = months.get(texts[index + 1] ?? "");
  if (first !== null && monthAfter !== undefined)
    return dated(first, monthAfter, index + 2);
  const month = months.get(texts[index]);
  if (month !== undefined) {
    const day = dayAt(index + 1);
    if (day !== null) return dated(day, month, index + 2);
    const year = yearAt(index + 1);
    return { facet: { kind: "month", month, year }, length: year ? 2 : 1 };
  }
  const year = yearAt(index);
  return year === null ? null : { facet: { kind: "year", year }, length: 1 };
}
