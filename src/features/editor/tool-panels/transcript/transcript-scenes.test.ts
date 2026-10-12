// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { SceneOffer } from "../scene/scene-offer";

import {
  transcriptSceneChoice,
  transcriptSceneTarget,
  transcriptTimelineSpan,
} from "./transcript-scenes";
import { TranscriptUnit } from "./transcript-units";

const unit = (
  startMs: number,
  endMs: number,
  removed = false,
): TranscriptUnit => ({
  corrected: false,
  endMs,
  heard: "word",
  kind: "word",
  removed,
  startMs,
  text: "word",
  unsure: false,
});

const offer = (canPlaceCamera: boolean): SceneOffer => ({
  canPlaceCamera,
  isCombined: canPlaceCamera,
  presets: canPlaceCamera ? ["stacked"] : [],
  templates: [],
});

describe("transcriptTimelineSpan", () => {
  it("places the recording on the edited video past cuts and at each speed", () => {
    // 0-1000 plays as is, 1000-2000 is cut, 2000-4000 plays at double speed.
    const played = [
      { endMs: 1000, startMs: 0 },
      { endMs: 4000, rate: 2, startMs: 2000 },
    ];
    expect(
      transcriptTimelineSpan(played, [{ endMs: 3000, startMs: 500 }]),
    ).toEqual({ lengthMs: 2000, span: { endMs: 1500, startMs: 500 } });
  });

  it("spans from the first moment played to the last on a reordered video", () => {
    const played = [
      { endMs: 2000, startMs: 1000 },
      { endMs: 1000, startMs: 0 },
    ];
    expect(
      transcriptTimelineSpan(played, [
        { endMs: 600, startMs: 400 },
        { endMs: 1200, startMs: 1100 },
      ]),
    ).toEqual({ lengthMs: 2000, span: { endMs: 1600, startMs: 100 } });
  });

  it("has no span where nothing chosen plays", () => {
    expect(
      transcriptTimelineSpan(
        [{ endMs: 1000, startMs: 0 }],
        [{ endMs: 1500, startMs: 1200 }],
      ).span,
    ).toBeNull();
  });
});

describe("transcriptSceneTarget", () => {
  const played = [{ endMs: 10_000, startMs: 0 }];

  it("leaves struck words out of the span", () => {
    expect(
      transcriptSceneTarget({
        offer: offer(true),
        played,
        scenes: [],
        span: { first: 0, last: 2 },
        units: [unit(1000, 1600), unit(1600, 2000), unit(2000, 4000, true)],
      }),
    ).toEqual({ endMs: 2000, startMs: 1000, type: "add" });
  });

  it("does not change a scene paused for want of a camera", () => {
    const scenes = [
      { endMs: 3000, id: "a", needsCamera: true, startMs: 0 },
      { endMs: 6000, id: "b", needsCamera: false, startMs: 3000 },
    ];
    const target = (first: number) =>
      transcriptSceneTarget({
        offer: offer(false),
        played,
        scenes,
        span: { first, last: first },
        units: [unit(1000, 1500), unit(4000, 4500)],
      });
    expect(target(0)).toBeNull();
    expect(target(1)).toEqual({
      endMs: 4500,
      id: "b",
      startMs: 4000,
      type: "change",
    });
  });
});

describe("transcriptSceneChoice", () => {
  it("only names a layout still on offer", () => {
    expect(transcriptSceneChoice(offer(true), "stacked")).toEqual({
      preset: "stacked",
      type: "preset",
    });
    expect(transcriptSceneChoice(offer(false), "stacked")).toBeNull();
    expect(transcriptSceneChoice(offer(false), "custom")).toEqual({
      type: "custom",
    });
    expect(transcriptSceneChoice(offer(true), "template:gone")).toBeNull();
  });
});
