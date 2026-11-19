// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { describe, expect, it } from "vitest";

import { playsAcross } from "./use-recording-link";

describe("the teleprompter with a recording", () => {
  it("plays once the countdown ends, not while it runs", () => {
    expect(playsAcross("idle", "starting")).toBeNull();
    expect(playsAcross("starting", "recording")).toBe(true);
  });

  it("pauses and resumes with the recording", () => {
    expect(playsAcross("recording", "paused")).toBe(false);
    expect(playsAcross("paused", "recording")).toBe(true);
  });

  it("pauses as the recording stops, and leaves it be after", () => {
    expect(playsAcross("recording", "stopping")).toBe(false);
    expect(playsAcross("stopping", "idle")).toBeNull();
  });

  it("leaves a paused teleprompter paused when a paused recording stops", () => {
    expect(playsAcross("paused", "stopping")).toBeNull();
  });
});
