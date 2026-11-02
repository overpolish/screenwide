// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The words a search reads dates in, in the form they are compared in. A
// language's weekday and month names come from its own calendar through
// `Intl`, so every translation reads them without a translator listing them;
// the rest are lists translators supply (`project-browser-date-words-…`).

/** The folded words of each phrase that means one thing ("last week"). */
export type Phrases = readonly (readonly string[])[];

/**
 * `text` as it is compared: lowercased as `locale` lowercases, accents and
 * other marks dropped, and stops dropped (so "Oct." and "a.m." read as
 * "oct" and "am"), except between digits ("15.00"). Commas go too, so
 * "Monday, Oct 27" reads as its words.
 */
export const fold = (text: string, locale: string) =>
  text
    .normalize("NFD")
    .replace(/\p{M}/gu, "")
    .toLocaleLowerCase(locale)
    .replace(/(?<!\d)\.|\.(?!\d)|,/gu, "");

/** A translator's list, words or phrases separated by commas, folded. */
export const phraseList = (list: string, locale: string): Phrases =>
  list
    .split(",")
    .map((phrase) => fold(phrase, locale).split(/\s+/u).filter(Boolean))
    .filter((words) => words.length > 0);

// Each name is read in every form the calendar writes it: alone and inside
// a date, since some languages inflect a month there ("октября" beside
// "октябрь"), each long and short. Formatted in UTC, so the day shown is
// the one meant wherever the machine is.
const namesOf = (
  locale: string,
  part: "month" | "weekday",
  dates: readonly Date[],
) => {
  const styles = ["long", "short"] as const;
  const formats = styles.flatMap((style): Intl.DateTimeFormatOptions[] =>
    part === "month"
      ? [
          { month: style, timeZone: "UTC" },
          { day: "numeric", month: style, timeZone: "UTC" },
        ]
      : [
          { timeZone: "UTC", weekday: style },
          { day: "numeric", month: "long", timeZone: "UTC", weekday: style },
        ],
  );
  const formatters = formats.map(
    (options) => new Intl.DateTimeFormat(locale, options),
  );
  return dates.map(
    (date) =>
      new Set(
        formatters.flatMap((formatter) =>
          formatter
            .formatToParts(date)
            .filter(({ type }) => type === part)
            .map(({ value }) => fold(value, locale)),
        ),
      ),
  );
};

/**
 * The names `locale` writes months and weekdays in, folded: months by index
 * (0 for January), weekdays as `Date.getDay` numbers them (0 for Sunday). A
 * name that could mean two things (Spanish "mar", both March and Tuesday) is
 * left out of both, as it cannot be told which is meant.
 */
export function calendarNames(locale: string) {
  const months = namesOf(
    locale,
    "month",
    Array.from(
      { length: 12 },
      (_, month) => new Date(Date.UTC(2024, month, 15)),
    ),
  );
  // 7 January 2024 was a Sunday.
  const weekdays = namesOf(
    locale,
    "weekday",
    Array.from({ length: 7 }, (_, day) => new Date(Date.UTC(2024, 0, 7 + day))),
  );
  const counts = new Map<string, number>();
  for (const names of [...months, ...weekdays])
    for (const name of names) counts.set(name, (counts.get(name) ?? 0) + 1);
  const byName = (all: Set<string>[]) =>
    new Map(
      all.flatMap((names, index) =>
        [...names]
          .filter((name) => name !== "" && counts.get(name) === 1)
          .map((name) => [name, index] as const),
      ),
    );
  return { months: byName(months), weekdays: byName(weekdays) };
}

type WeekInfo = { firstDay: number };

/** The day `locale`'s weeks start on, as `Date.getDay` numbers it; Monday
 * where the engine cannot say, as most of the world starts there. */
export function weekStartOf(locale: string) {
  const found = new Intl.Locale(locale) as Intl.Locale & {
    getWeekInfo?: () => WeekInfo;
    weekInfo?: WeekInfo;
  };
  const info = found.getWeekInfo?.() ?? found.weekInfo;
  // `Intl` numbers Monday 1 to Sunday 7.
  return (info?.firstDay ?? 1) % 7;
}
