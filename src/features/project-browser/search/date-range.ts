// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The span of time a date read from a search covers. Days start at local
// midnight and are counted on the calendar, so a daylight-saving change never
// moves one.

export type DayPart = "afternoon" | "evening" | "morning" | "night";
export type Period = "last-month" | "last-week" | "this-month" | "this-week";

type DateReading =
  | {
      hour: number | null;
      kind: "day";
      part: DayPart | null;
      /** Said as today or yesterday rather than as a date. */
      relative: "today" | "yesterday" | null;
    }
  | { kind: "month" }
  | { kind: "period"; period: Period }
  | { count: number; kind: "recent"; unit: "hour" | "minute" }
  | { kind: "year" };

/** What a date read from a search covers, from `start` up to, not including,
 * `end`, in milliseconds since the epoch, and how it was read. */
export type DateRange = { end: number; reading: DateReading; start: number };

/** Hours of the day each part spans, night running into the next morning. */
export const DAY_PARTS: Record<DayPart, readonly [number, number]> = {
  afternoon: [12, 17],
  evening: [17, 21],
  morning: [5, 12],
  night: [21, 29],
};

/** One thing a query names: a day, a time of day, or a span alone. */
export type Facet =
  | { day: number; kind: "date"; month: number; year: number | null }
  | { hour: number; kind: "hour" }
  | { kind: "month"; month: number; year: number | null }
  | { kind: "part"; part: DayPart }
  | { kind: "period"; period: Period }
  | { count: number; kind: "recent"; unit: "hour" | "minute" }
  | { days: 0 | 1; kind: "relative" }
  | { kind: "weekday"; weekday: number }
  | { kind: "year"; year: number };

const HOUR_MS = 60 * 60 * 1_000;

/** The range the facets read together cover: a day with at most one time
 * of day, a time of day alone (its latest that has begun), or one of the
 * rest alone. */
export function resolveRange(
  facets: readonly Facet[],
  now: Date,
  weekStart: number,
): DateRange | null {
  const days = facets.filter(({ kind }) =>
    ["date", "relative", "weekday"].includes(kind),
  );
  const times = facets.filter(({ kind }) => ["hour", "part"].includes(kind));
  const rest = facets.length - days.length - times.length;
  if (days.length > 1 || times.length > 1 || (rest > 0 && facets.length > 1))
    return null;
  const [day] = days as (Facet | undefined)[];
  const [time] = times as (Facet | undefined)[];
  const year = now.getFullYear();
  const month = now.getMonth();
  const date = now.getDate();
  if (!day && !time) return alone(facets[0], now, weekStart);

  const [startHour, endHour] =
    time?.kind === "hour"
      ? [time.hour, time.hour + 1]
      : time?.kind === "part"
        ? DAY_PARTS[time.part]
        : [0, 24];
  let on: Date | null;
  let relative: "today" | "yesterday" | null = null;
  if (!day) {
    // A time alone is its latest that has begun: 3pm read at 10am is
    // yesterday's.
    const isToday = new Date(year, month, date, startHour) <= now;
    on = new Date(year, month, isToday ? date : date - 1);
    relative = isToday ? "today" : "yesterday";
  } else if (day.kind === "relative") {
    on = new Date(year, month, date - day.days);
    relative = day.days === 0 ? "today" : "yesterday";
  } else if (day.kind === "weekday") {
    on = new Date(year, month, date - ((now.getDay() - day.weekday + 7) % 7));
  } else {
    on = latestDate(day as Extract<Facet, { kind: "date" }>, now);
  }
  if (!on) return null;
  const at = (hour: number) =>
    new Date(on.getFullYear(), on.getMonth(), on.getDate(), hour).getTime();
  return {
    end: at(endHour),
    reading: {
      hour: time?.kind === "hour" ? time.hour : null,
      kind: "day",
      part: time?.kind === "part" ? time.part : null,
      relative,
    },
    start: at(startHour),
  };
}

/** The day and month in the year given, else their latest that has begun;
 * null for a day the month does not have. */
function latestDate(
  { day, month, year }: Extract<Facet, { kind: "date" }>,
  now: Date,
) {
  const thisYear = new Date(now.getFullYear(), month, day);
  const on =
    year !== null
      ? new Date(year, month, day)
      : thisYear <= now
        ? thisYear
        : new Date(now.getFullYear() - 1, month, day);
  return on.getMonth() === month ? on : null;
}

function alone(facet: Facet, now: Date, weekStart: number): DateRange | null {
  const year = now.getFullYear();
  const month = now.getMonth();
  switch (facet.kind) {
    case "recent": {
      const span = facet.unit === "hour" ? HOUR_MS : HOUR_MS / 60;
      // Through now and on, so what is recorded after the query is read
      // still counts.
      return {
        end: Number.POSITIVE_INFINITY,
        reading: facet,
        start: now.getTime() - facet.count * span,
      };
    }
    case "month": {
      const inYear = facet.year ?? (facet.month <= month ? year : year - 1);
      return {
        end: new Date(inYear, facet.month + 1, 1).getTime(),
        reading: { kind: "month" },
        start: new Date(inYear, facet.month, 1).getTime(),
      };
    }
    case "year":
      return {
        end: new Date(facet.year + 1, 0, 1).getTime(),
        reading: { kind: "year" },
        start: new Date(facet.year, 0, 1).getTime(),
      };
    case "period": {
      const back = facet.period.startsWith("last") ? 1 : 0;
      if (facet.period.endsWith("month"))
        return {
          end: new Date(year, month - back + 1, 1).getTime(),
          reading: facet,
          start: new Date(year, month - back, 1).getTime(),
        };
      const weekBegan =
        now.getDate() - ((now.getDay() - weekStart + 7) % 7) - back * 7;
      return {
        end: new Date(year, month, weekBegan + 7).getTime(),
        reading: facet,
        start: new Date(year, month, weekBegan).getTime(),
      };
    }
    default:
      return null;
  }
}
