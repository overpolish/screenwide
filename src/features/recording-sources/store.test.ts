// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { mergeRecordingSourceState } from "./store";

describe("mergeRecordingSourceState", () => {
  it("clears a stale linked ratio when persisted JSON omits it", () => {
    const current = { regionAspectRatio: 16 / 9, regionId: "current" };

    expect(
      mergeRecordingSourceState({ regionId: "persisted" }, current),
    ).toEqual({
      regionAspectRatio: undefined,
      regionId: "persisted",
    });
  });

  it("hydrates a persisted linked ratio", () => {
    const current = { regionAspectRatio: undefined };

    expect(
      mergeRecordingSourceState({ regionAspectRatio: 4 / 3 }, current),
    ).toEqual({ regionAspectRatio: 4 / 3 });
  });
});
