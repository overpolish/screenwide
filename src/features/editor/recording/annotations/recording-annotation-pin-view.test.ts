// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { pinUnsureAt, recordingPinView } from "./recording-annotation-pin-view";
import { RecordingAnnotationPin } from "./recording-annotations";
import { RecordingPinStatus } from "./use-recording-pin-status";

const pin: RecordingAnnotationPin = {
  keyframes: [{ dx: 0, dy: 0, ms: 2_000 }],
  pinnedMs: 2_000,
};
const landed: RecordingPinStatus = {
  covered: [],
  hidden: [],
  progress: null,
  under: [],
  weak: [],
};
const at = (
  status: RecordingPinStatus | undefined,
  { canHide = false, canShow = false, of = pin, positionMs = 5_000 } = {},
) => recordingPinView({ canHide, canShow, pin: of, positionMs, status });

describe("a pinned annotation's island", () => {
  it("is tracking until the tracker says the path has landed", () => {
    expect(at(undefined)).toMatchObject({ progress: null, state: "tracking" });
    expect(at({ ...landed, progress: 0.4 })).toMatchObject({
      progress: 0.4,
      state: "tracking",
    });
  });

  it("follows the content once landed, offering to say it went out of view", () => {
    expect(at(landed, { canHide: true })).toEqual({
      action: "hide",
      hasCorrections: false,
      progress: null,
      state: "following",
      unsure: null,
    });
  });

  it("says the content is out of view where it is hidden, offering it back", () => {
    expect(at(landed, { canShow: true })).toMatchObject({
      action: "show",
      state: "out-of-view",
    });
  });

  it("offers nothing to say where the playhead is outside it", () => {
    expect(at(landed).action).toBeNull();
  });

  it("has corrections once put anywhere by hand besides its pinned frame", () => {
    const corrected = {
      ...pin,
      keyframes: [...pin.keyframes, { dx: 4, dy: 0, ms: 3_000 }],
    };
    expect(at(landed).hasCorrections).toBe(false);
    expect(at(landed, { of: corrected }).hasCorrections).toBe(true);
  });
});

describe("where the tracker is unsure of a pinned annotation's content", () => {
  const unsure: RecordingPinStatus = {
    ...landed,
    hidden: [[3_000, 5_000]],
    under: [[6_000, 8_000]],
    weak: [
      [1_000, 2_000],
      [4_000, 4_500],
    ],
  };

  it("says why at the playhead, each kind from its own stretches", () => {
    expect(pinUnsureAt(unsure, 1_500)).toBe("lost");
    expect(pinUnsureAt(unsure, 3_000)).toBe("off-frame");
    expect(pinUnsureAt(unsure, 7_999)).toBe("under-cover");
  });

  it("is sure between the stretches and past their ends", () => {
    expect(pinUnsureAt(unsure, 2_000)).toBeNull();
    expect(pinUnsureAt(unsure, 5_500)).toBeNull();
    expect(pinUnsureAt(unsure, 8_000)).toBeNull();
  });

  it("says lost first where the content was also off the frame", () => {
    expect(pinUnsureAt(unsure, 4_200)).toBe("lost");
  });

  it("says nothing before the path has landed", () => {
    expect(pinUnsureAt(undefined, 1_500)).toBeNull();
    expect(pinUnsureAt({ ...unsure, progress: 0.5 }, 1_500)).toBeNull();
  });

  it("warns on the island, keeping the answer the playhead allows", () => {
    expect(at(unsure, { canHide: true, positionMs: 1_500 })).toMatchObject({
      action: "hide",
      state: "following",
      unsure: "lost",
    });
    expect(at(unsure, { canShow: true, positionMs: 7_000 })).toMatchObject({
      action: "show",
      state: "out-of-view",
      unsure: "under-cover",
    });
  });

  it("says nothing where the playhead is outside the clip", () => {
    expect(at(unsure, { positionMs: 1_500 }).unsure).toBeNull();
  });
});
