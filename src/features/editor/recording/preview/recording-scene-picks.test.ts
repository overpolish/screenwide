// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { scenePanePicks } from "./recording-scene-picks";

import type { RecordingPreviewSelection } from "../use-recording-preview-surface";

const pane = (layerId: number): RecordingPreviewSelection => ({
  layerId,
  paneIndex: Math.min(layerId, 1),
  radiusPercent: 0,
  rect: { height: 0.5, width: 0.5, x: 0.25, y: 0.25 },
});

const picks = (isInGap: boolean) =>
  scenePanePicks({
    arrangement: null,
    canPreviewBakedCamera: true,
    freeCamera: false,
    freeScreen: false,
    isFramed: false,
    isInGap,
    overlay: pane(0),
    reframe: undefined,
    screenRotation: undefined,
    targets: [pane(0), pane(1), pane(2)],
  });

describe("the panes picked", () => {
  it("draw no screen or camera chrome in a gap, which shows neither", () => {
    const inGap = picks(true);
    expect(inGap.overlay).toBeNull();
    expect(inGap.targets?.map(({ layerId }) => layerId)).toEqual([2]);
  });

  it("keep the screen and camera outside a gap", () => {
    const outside = picks(false);
    expect(outside.overlay?.layerId).toBe(0);
    expect(outside.targets?.map(({ layerId }) => layerId)).toEqual([0, 1, 2]);
  });
});
