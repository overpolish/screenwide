// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  arrangedScenePane,
  scenePaneArrangements,
  swappedScenePanes,
} from "./recording-scene-order";
import { RecordingSceneClip } from "./recording-scenes";

const SCREEN = { height: 0.8, width: 0.8, x: 0.1, y: 0.1 };
const BUBBLE = { height: 0.25, width: 0.15, x: 0.75, y: 0.7 };

const bubble: RecordingSceneClip = {
  boxes: { camera: BUBBLE, screen: SCREEN },
  camera: { focusX: 0.4, focusY: 0.5, zoom: 1.5 },
  endMs: 5000,
  id: "b",
  preset: "picture-in-picture",
  radius: { camera: 50, screen: 4 },
  screen: { focusX: 0.5, focusY: 0.5, zoom: 1 },
  startMs: 0,
};

describe("swappedScenePanes", () => {
  it("puts the small screen over the big camera, each keeping its own crop and corners", () => {
    const swapped = swappedScenePanes(bubble);
    expect(swapped.boxes).toEqual({
      camera: SCREEN,
      cameraBehind: true,
      screen: BUBBLE,
    });
    expect(swapped.camera).toEqual(bubble.camera);
    expect(swapped.screen).toEqual(bubble.screen);
    expect(swapped.radius).toEqual(bubble.radius);
  });

  it("gives the scene back when swapped again", () => {
    expect(swappedScenePanes(swappedScenePanes(bubble))).toEqual(bubble);
  });

  it("leaves a scene without a box for each pane alone", () => {
    const screenOnly = { ...bubble, boxes: { screen: SCREEN } };
    expect(swappedScenePanes(screenOnly)).toBe(screenOnly);
  });
});

describe("arrangedScenePane", () => {
  it("sends the camera behind the screen, and either pane brings it back", () => {
    const behind = arrangedScenePane(bubble, "camera", "backward");
    expect(behind.boxes?.cameraBehind).toBe(true);
    expect(scenePaneArrangements(behind, "camera")).toEqual({
      canBringForward: true,
      canSendBackward: false,
    });
    expect(arrangedScenePane(behind, "screen", "back")).toEqual(bubble);
    expect(arrangedScenePane(behind, "camera", "front")).toEqual(bubble);
  });

  it("offers no move for a scene that does not order its panes", () => {
    const preset: RecordingSceneClip = { ...bubble, boxes: undefined };
    expect(scenePaneArrangements(preset, "camera")).toEqual({
      canBringForward: false,
      canSendBackward: false,
    });
    expect(arrangedScenePane(preset, "screen", "front")).toBe(preset);
  });
});
