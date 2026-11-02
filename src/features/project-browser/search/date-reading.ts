// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { appLocale, sourceText, t } from "../../../i18n/i18n";

import { calendarNames, phraseList, weekStartOf } from "./date-names";

import type { DateVocabulary, Keyword } from "./date-query";
import type { DateRange, Period } from "./date-range";

// The app's language for reading dates out of a search, and for saying back
// how one was read, so the person searching sees they were understood.

const KEYWORDS = {
  afternoon: "project-browser-date-words-afternoon",
  ago: "project-browser-date-words-ago",
  am: "project-browser-date-words-am",
  evening: "project-browser-date-words-evening",
  hours: "project-browser-date-words-hours",
  last: "project-browser-date-words-last",
  "last-month": "project-browser-date-words-last-month",
  "last-week": "project-browser-date-words-last-week",
  minutes: "project-browser-date-words-minutes",
  morning: "project-browser-date-words-morning",
  night: "project-browser-date-words-night",
  one: "project-browser-date-words-one",
  pm: "project-browser-date-words-pm",
  "this-month": "project-browser-date-words-this-month",
  "this-week": "project-browser-date-words-this-week",
  today: "project-browser-date-words-today",
  yesterday: "project-browser-date-words-yesterday",
} as const satisfies Record<Keyword, string>;

const PERIODS = {
  "last-month": "project-browser-date-last-month",
  "last-week": "project-browser-date-last-week",
  "this-month": "project-browser-date-this-month",
  "this-week": "project-browser-date-this-week",
} as const satisfies Record<Period, string>;

let cached: { key: string; vocabulary: DateVocabulary } | null = null;

/** The words dates are read in: the language's calendar names, and its
 * translators' date words with the source's beside them. Built once per
 * language, as it takes some hundred formats. */
export function dateVocabulary(): DateVocabulary {
  const { formatLocale, language } = appLocale();
  const key = `${language} ${formatLocale}`;
  if (cached?.key === key) return cached.vocabulary;
  const words = Object.fromEntries(
    Object.entries(KEYWORDS).map(([keyword, id]) => [
      keyword,
      [
        ...phraseList(t(id), formatLocale),
        ...phraseList(sourceText(id), formatLocale),
      ],
    ]),
  ) as Record<Keyword, string[][]>;
  const vocabulary: DateVocabulary = {
    locale: formatLocale,
    ...calendarNames(formatLocale),
    weekStart: weekStartOf(formatLocale),
    words,
  };
  cached = { key, vocabulary };
  return vocabulary;
}

/** How a date read from a search is said back: "Recorded yesterday",
 * "Recorded Monday, Oct 27", "Recorded in the last hour". The year is named
 * only when it is not this one. */
export function dateReadingLabel(
  { end, reading, start }: DateRange,
  now: Date,
): string {
  const { formatLocale } = appLocale();
  const isThisYear = new Date(start).getFullYear() === now.getFullYear();
  const year = isThisYear ? {} : ({ year: "numeric" } as const);
  const dayOptions = {
    day: "numeric",
    month: "short",
    weekday: "long",
    ...year,
  } as const;
  const day = new Intl.DateTimeFormat(formatLocale, dayOptions);
  switch (reading.kind) {
    case "day": {
      const { hour, part, relative } = reading;
      if (part)
        return t("project-browser-date-part", {
          date: day.format(start),
          day: relative ?? "other",
          part,
        });
      if (hour !== null) {
        const hours = new Intl.DateTimeFormat(formatLocale, {
          hour: "numeric",
          ...(relative ? {} : dayOptions),
        }).formatRange(start, end);
        if (relative === "today")
          return t("project-browser-date-today-hours", { hours });
        if (relative === "yesterday")
          return t("project-browser-date-yesterday-hours", { hours });
        return t("project-browser-date-on", { date: hours });
      }
      if (relative === "today") return t("project-browser-date-today");
      if (relative === "yesterday") return t("project-browser-date-yesterday");
      return t("project-browser-date-on", { date: day.format(start) });
    }
    case "month":
      return t("project-browser-date-in", {
        period: new Intl.DateTimeFormat(formatLocale, {
          month: "long",
          ...year,
        }).format(start),
      });
    case "year":
      return t("project-browser-date-in", {
        period: new Intl.DateTimeFormat(formatLocale, {
          year: "numeric",
        }).format(start),
      });
    case "period":
      return t(PERIODS[reading.period]);
    case "recent":
      return t("project-browser-date-recent", {
        count: reading.count,
        unit: reading.unit,
      });
  }
}
