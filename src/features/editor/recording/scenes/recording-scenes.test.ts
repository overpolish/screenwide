// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  insertRecordingSceneClip,
  RecordingSceneClip,
} from "./recording-scenes";

const DURATION = 60_000;
const clip = (id: string, startMs: number, endMs: number) =>
  ({ endMs, id, preset: "split-two-thirds", startMs }) as RecordingSceneClip;
const insert = (clips: RecordingSceneClip[], atMs: number) =>
  insertRecordingSceneClip({
    atMs,
    clips,
    id: "new",
    lengthMs: DURATION,
    preset: "split-two-thirds",
  });

describe("insertRecordingSceneClip", () => {
  it("runs five seconds from the playhead where there is room", () => {
    expect(insert([], 10_000)?.clip).toMatchObject({
      endMs: 15_000,
      startMs: 10_000,
    });
  });

  it("stops at the next clip and reaches back for the rest", () => {
    const next = insert(
      [clip("a", 0, 8_000), clip("b", 12_000, 20_000)],
      11_000,
    );
    expect(next?.clip).toMatchObject({ endMs: 12_000, startMs: 8_000 });
    expect(next?.clips.map((item) => item.id)).toEqual(["a", "new", "b"]);
  });

  it("reaches back from the end of the recording", () => {
    expect(insert([], 59_000)?.clip).toMatchObject({
      endMs: 60_000,
      startMs: 55_000,
    });
  });

  it("adds nothing inside a clip or where the gap is too short", () => {
    expect(insert([clip("a", 0, 8_000)], 4_000)).toBeNull();
    expect(
      insert([clip("a", 0, 8_000), clip("b", 8_200, 20_000)], 8_100),
    ).toBeNull();
  });
});
