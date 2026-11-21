// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultScreenshotOutput } from "../../screenshot/screenshot-output";

import { recordingSceneEdits } from "./recording-scene-edits";
import { RecordingSceneClip } from "./recording-scenes";

const CLIPS: RecordingSceneClip[] = [
  { endMs: 3_000, id: "first", preset: "full", startMs: 1_000 },
  { endMs: 9_000, id: "second", preset: "full", startMs: 7_000 },
];

/** The Scene panel's edits with `chosenId` chosen on the strip and the
 * playhead at `atMs`, and the clips each ask commits. */
const editsAt = (atMs: number, chosenId: string | null) => {
  const committed: RecordingSceneClip[][] = [];
  const edits = recordingSceneEdits({
    camera: null,
    chosenId,
    clips: CLIPS,
    commit: (next) => committed.push(next),
    getTimelineMs: () => atMs,
    lengthMs: 10_000,
    output: defaultScreenshotOutput(1600, 900),
    paneOf: () => null,
  });
  return { committed, edits };
};

describe("the Scene panel's asks", () => {
  it("give the chosen scene a layout wherever the playhead is, adding none", () => {
    const { committed, edits } = editsAt(5_000, "second");
    edits.choosePreset("screen-only");
    expect(committed).toEqual([
      [CLIPS[0], { ...CLIPS[1], preset: "screen-only" }],
    ]);
  });

  it("act on the scene under the playhead with none chosen", () => {
    const { committed, edits } = editsAt(2_000, null);
    edits.choosePreset("blank");
    expect(committed).toEqual([[{ ...CLIPS[0], preset: "blank" }, CLIPS[1]]]);
  });

  it("add a scene at the playhead with none chosen or under it", () => {
    const { committed, edits } = editsAt(5_000, null);
    edits.choosePreset("blank");
    expect(committed[0]).toHaveLength(3);
    expect(committed[0]?.[1]).toMatchObject({ preset: "blank" });
  });

  it("take away the chosen scene, not the one under the playhead", () => {
    const { committed, edits } = editsAt(2_000, "second");
    edits.remove();
    expect(committed).toEqual([[CLIPS[0]]]);
  });

  it("crop the scene the picture shows, under the playhead", () => {
    const { committed, edits } = editsAt(5_000, "second");
    edits.cropScreen("1:1");
    expect(committed).toEqual([]);
  });
});
