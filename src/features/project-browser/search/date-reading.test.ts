// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { withoutIsolates } from "../../../i18n/testing";

import { parseDateQuery } from "./date-query";
import { dateReadingLabel, dateVocabulary } from "./date-reading";

// Monday 2 November 2026, mid-morning local time.
const now = new Date(2026, 10, 2, 10, 0);

/** How `query` is said back, with the formatters' narrow spaces as plain. */
const label = (query: string) => {
  const found = parseDateQuery(query, now, dateVocabulary());
  if (!found) throw new Error(`${query} read as no date`);
  return withoutIsolates(dateReadingLabel(found, now)).replace(/\s/gu, " ");
};

describe("dateReadingLabel", () => {
  it.each([
    ["today", "Recorded today"],
    ["yesterday", "Recorded yesterday"],
    ["tuesday", "Recorded Tuesday, Oct 27"],
    ["12 oct 2025", "Recorded Sunday, Oct 12, 2025"],
    ["yesterday morning", "Recorded yesterday morning"],
    ["morning", "Recorded this morning"],
    ["night", "Recorded last night"],
    ["tuesday evening", "Recorded Tuesday, Oct 27 in the evening"],
    ["3pm", "Recorded yesterday, 3 – 4 PM"],
    ["tuesday 3pm", "Recorded Tuesday, Oct 27, 3 – 4 PM"],
    ["october", "Recorded in October"],
    ["dec", "Recorded in December 2025"],
    ["2025", "Recorded in 2025"],
    ["this week", "Recorded this week"],
    ["last month", "Recorded last month"],
    ["an hour ago", "Recorded in the last hour"],
    ["2 hours ago", "Recorded in the last 2 hours"],
    ["30 minutes ago", "Recorded in the last 30 minutes"],
  ])("says %s as %s", (query, expected) => {
    expect(label(query)).toBe(expected);
  });
});
