// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { calendarNames } from "./date-names";
import { parseDateQuery, type DateVocabulary } from "./date-query";
import { dateVocabulary } from "./date-reading";

const english = dateVocabulary();

// Monday 2 November 2026, mid-morning local time.
const now = new Date(2026, 10, 2, 10, 0);

const at = (...parts: [number, number, number, number?, number?]) =>
  new Date(...parts).getTime();

const read = (
  query: string,
  when: Date = now,
  vocabulary: DateVocabulary = english,
) => parseDateQuery(query, when, vocabulary);

/** The range read, as local times, and the words left. */
const span = (query: string, when?: Date, vocabulary?: DateVocabulary) => {
  const found = read(query, when, vocabulary);
  return found && [found.start, found.end, found.words];
};

const inLocale = (locale: string): DateVocabulary => ({
  ...english,
  locale,
  ...calendarNames(locale),
});

describe("parseDateQuery days", () => {
  it("reads today and yesterday from local midnight", () => {
    expect(span("today")).toEqual([at(2026, 10, 2), at(2026, 10, 3), ""]);
    expect(span("Yesterday")).toEqual([at(2026, 10, 1), at(2026, 10, 2), ""]);
    expect(read("yesterday")?.reading).toEqual({
      hour: null,
      kind: "day",
      part: null,
      relative: "yesterday",
    });
  });

  it("is yesterday's at a minute past midnight, not before", () => {
    const justAfter = new Date(2026, 10, 2, 0, 1);
    expect(span("yesterday", justAfter)?.slice(0, 2)).toEqual([
      at(2026, 10, 1),
      at(2026, 10, 2),
    ]);
  });

  it("reads a weekday as its latest, today included", () => {
    expect(span("monday")).toEqual([at(2026, 10, 2), at(2026, 10, 3), ""]);
    expect(span("sunday")?.[0]).toBe(at(2026, 10, 1));
    expect(span("TUE")?.[0]).toBe(at(2026, 9, 27));
    expect(span("tuesday")?.[0]).toBe(at(2026, 9, 27));
    expect(read("monday")?.reading).toMatchObject({ relative: null });
  });
});

describe("parseDateQuery calendar", () => {
  it("reads a month as its latest that has begun", () => {
    expect(span("october")).toEqual([at(2026, 9, 1), at(2026, 10, 1), ""]);
    expect(span("November")?.[0]).toBe(at(2026, 10, 1));
    expect(span("dec")?.slice(0, 2)).toEqual([at(2025, 11, 1), at(2026, 0, 1)]);
    expect(span("october 2025")?.[0]).toBe(at(2025, 9, 1));
  });

  it("reads a day and month in either order, short or long", () => {
    for (const query of ["12 oct", "oct 12", "Oct. 12", "12 October"])
      expect(span(query)).toEqual([at(2026, 9, 12), at(2026, 9, 13), ""]);
    expect(span("12 december")?.[0]).toBe(at(2025, 11, 12));
    expect(span("oct 12 2025")?.[0]).toBe(at(2025, 9, 12));
    expect(span("12oct")?.[0]).toBe(at(2026, 9, 12));
  });

  it("reads a year up to this one", () => {
    expect(span("2025")).toEqual([at(2025, 0, 1), at(2026, 0, 1), ""]);
    expect(read("2030")).toBeNull();
    expect(read("1080")).toBeNull();
  });

  it("leaves a day a month does not have as words", () => {
    expect(span("feb 30")).toBeNull();
    expect(span("31 sep pricing")).toBeNull();
  });
});

describe("parseDateQuery weeks and months", () => {
  it("starts the week where the language does", () => {
    // en-US weeks start on Sunday, so Monday's week began yesterday.
    expect(span("this week")).toEqual([at(2026, 10, 1), at(2026, 10, 8), ""]);
    expect(span("last week")).toEqual([at(2026, 9, 25), at(2026, 10, 1), ""]);
    const mondays = { ...english, weekStart: 1 };
    expect(span("this week", now, mondays)?.[0]).toBe(at(2026, 10, 2));
    expect(span("last week", now, mondays)?.[0]).toBe(at(2026, 9, 26));
  });

  it("reads this month and last", () => {
    expect(span("this month")).toEqual([at(2026, 10, 1), at(2026, 11, 1), ""]);
    expect(span("last month")).toEqual([at(2026, 9, 1), at(2026, 10, 1), ""]);
    const january = new Date(2027, 0, 5);
    expect(span("last month", january)?.[0]).toBe(at(2026, 11, 1));
  });
});

describe("parseDateQuery recent spans", () => {
  const hour = 60 * 60 * 1_000;
  it.each([
    ["an hour ago", 1, "hour"],
    ["last hour", 1, "hour"],
    ["past hour", 1, "hour"],
    ["2 hours ago", 2, "hour"],
    ["2h ago", 2, "hour"],
    ["last 3 hours", 3, "hour"],
    ["30 minutes ago", 30, "minute"],
    ["a minute ago", 1, "minute"],
    ["past 15 min", 15, "minute"],
  ])("reads %s as the span up to now", (query, count, unit) => {
    const found = read(query);
    expect(found?.reading).toEqual({ count, kind: "recent", unit });
    expect(found?.start).toBe(
      now.getTime() - count * (unit === "hour" ? hour : hour / 60),
    );
    expect(found?.end).toBe(Number.POSITIVE_INFINITY);
  });

  it("does not read a number of hours alone", () => {
    expect(read("2 hours")).toBeNull();
    expect(read("hours")).toBeNull();
  });
});

describe("parseDateQuery times of day", () => {
  it("reads an hour as its latest that has begun", () => {
    // 3pm read at 10am is yesterday's.
    expect(span("3pm")).toEqual([at(2026, 10, 1, 15), at(2026, 10, 1, 16), ""]);
    expect(read("3pm")?.reading).toMatchObject({
      hour: 15,
      relative: "yesterday",
    });
    const evening = new Date(2026, 10, 2, 16, 30);
    for (const query of ["3 pm", "3 p.m.", "15:00", "15:30", "15.00"])
      expect(span(query, evening)?.[0]).toBe(at(2026, 10, 2, 15));
    expect(span("12am")?.[0]).toBe(at(2026, 10, 2, 0));
    expect(span("12 pm", evening)?.[0]).toBe(at(2026, 10, 2, 12));
  });

  it("leaves numbers that are no time as words", () => {
    expect(read("13pm")).toBeNull();
    expect(read("25:00")).toBeNull();
    expect(read("15")).toBeNull();
  });

  it("reads parts of the day, night into the next morning", () => {
    expect(span("morning")).toEqual([
      at(2026, 10, 2, 5),
      at(2026, 10, 2, 12),
      "",
    ]);
    expect(span("night")).toEqual([
      at(2026, 10, 1, 21),
      at(2026, 10, 2, 5),
      "",
    ]);
    expect(span("evening")?.[0]).toBe(at(2026, 10, 1, 17));
    expect(read("afternoon")?.reading).toMatchObject({
      part: "afternoon",
      relative: "yesterday",
    });
  });

  it("puts a time on a day", () => {
    expect(span("yesterday morning")?.slice(0, 2)).toEqual([
      at(2026, 10, 1, 5),
      at(2026, 10, 1, 12),
    ]);
    // Monday is today, its 3pm still to come.
    expect(span("monday 3pm")?.slice(0, 2)).toEqual([
      at(2026, 10, 2, 15),
      at(2026, 10, 2, 16),
    ]);
    expect(span("3pm monday")?.[0]).toBe(at(2026, 10, 2, 15));
    expect(span("oct 12 9am")?.[0]).toBe(at(2026, 9, 12, 9));
    expect(read("monday 3pm")?.reading).toEqual({
      hour: 15,
      kind: "day",
      part: null,
      relative: null,
    });
  });

  it("reads an hour before am or pm, not a day", () => {
    expect(span("may 3pm")).toBeNull();
  });
});

describe("parseDateQuery words", () => {
  it("leaves the words that are not the date, as typed", () => {
    expect(read("monday Pricing")?.words).toBe("Pricing");
    expect(read("Pricing yesterday call")?.words).toBe("Pricing call");
    expect(read("May launch")?.words).toBe("launch");
  });

  it("reads no date in plain words", () => {
    expect(read("launch")).toBeNull();
    expect(read("mayday")).toBeNull();
    expect(read("")).toBeNull();
  });

  it("reads no date named twice", () => {
    expect(read("monday tuesday")).toBeNull();
    expect(read("yesterday last week")).toBeNull();
    expect(read("3pm 4pm")).toBeNull();
    expect(read("october last week")).toBeNull();
    expect(read("2025 an hour ago")).toBeNull();
  });
});

describe("parseDateQuery in other languages", () => {
  it("reads the language's own weekday and month names", () => {
    const french = inLocale("fr-FR");
    expect(span("lundi", now, french)?.[0]).toBe(at(2026, 10, 2));
    expect(span("12 Octobre", now, french)?.[0]).toBe(at(2026, 9, 12));
    // Without the accent, and short.
    expect(span("decembre", now, french)?.[0]).toBe(at(2025, 11, 1));
    expect(span("3 févr.", now, french)?.[0]).toBe(at(2026, 1, 3));
    const german = inLocale("de-DE");
    expect(span("Marz", now, german)?.[0]).toBe(at(2026, 2, 1));
    expect(span("Dienstag", now, german)?.[0]).toBe(at(2026, 9, 27));
  });

  it("reads a month inflected inside a date", () => {
    const russian = inLocale("ru-RU");
    expect(span("октябрь", now, russian)?.[0]).toBe(at(2026, 9, 1));
    expect(span("12 октября", now, russian)?.[0]).toBe(at(2026, 9, 12));
  });

  it("leaves out a name that could be a month or a weekday", () => {
    const spanish = calendarNames("es-ES");
    expect(spanish.months.has("mar")).toBe(false);
    expect(spanish.weekdays.has("mar")).toBe(false);
    expect(spanish.months.get("marzo")).toBe(2);
    expect(spanish.weekdays.get("martes")).toBe(2);
  });
});
