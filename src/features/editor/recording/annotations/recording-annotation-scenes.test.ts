// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { Annotation } from "../../annotations/annotations";

import {
  annotationInScene,
  withEditInScene,
  withSceneShapesFollowing,
  withoutSceneShape,
} from "./recording-annotation-scenes";
import { RecordingAnnotationClip } from "./recording-annotations";

const text = (x: number, words: string): Annotation["shape"] => ({
  kind: "text",
  origin: { x, y: 0 },
  text: words,
});

const clip = (shape: Annotation["shape"]): RecordingAnnotationClip => ({
  annotation: {
    animated: true,
    id: "text-1",
    shape,
    style: {
      align: "left",
      blur: false,
      color: "#ff0000",
      handDrawn: false,
      head: "end",
      manual: false,
      radius: 0,
      redaction: "erase",
      shadow: false,
      softness: 0,
      strength: 0,
      tint: false,
      width: 8,
    },
  },
  endMs: 10_000,
  startMs: 0,
  trackId: "primary",
});

describe("an annotation's places in scenes", () => {
  it("keeps a move in a scene for that scene, and its words for every one", () => {
    const own = clip(text(0, "Hello"));
    const moved = {
      ...own.annotation,
      shape: text(400, "Hello"),
    };
    const placed = withEditInScene(own, moved, "scene-1");
    expect(placed.annotation.shape).toEqual(text(0, "Hello"));
    expect(annotationInScene(placed, "scene-1").shape).toEqual(
      text(400, "Hello"),
    );
    expect(annotationInScene(placed, "scene-2").shape).toEqual(
      text(0, "Hello"),
    );
    // Retyped in the scene, it says the new words everywhere and stays put.
    const retyped = withEditInScene(
      placed,
      { ...placed.annotation, shape: text(400, "Goodbye") },
      "scene-1",
    );
    expect(retyped.annotation.shape).toEqual(text(0, "Goodbye"));
    expect(annotationInScene(retyped, "scene-1").shape).toEqual(
      text(400, "Goodbye"),
    );
  });

  it("gives a scene nothing for an edit that leaves the annotation where it is", () => {
    const own = clip(text(0, "Hello"));
    const recoloured = withEditInScene(
      own,
      {
        ...own.annotation,
        style: { ...own.annotation.style, color: "#00ff00" },
      },
      "scene-1",
    );
    expect(recoloured.sceneShapes).toBeUndefined();
    expect(recoloured.annotation.style.color).toBe("#00ff00");
  });

  it("follows its scenes as they are cut in two and taken away", () => {
    const placed = withEditInScene(
      clip(text(0, "Hello")),
      { ...clip(text(0, "Hello")).annotation, shape: text(400, "Hello") },
      "scene-1",
    );
    const clips = [placed];
    expect(withSceneShapesFollowing(clips, ["scene-1"])).toBe(clips);
    const [split] = withSceneShapesFollowing(
      clips,
      ["scene-1", "scene-1b"],
      new Map([["scene-1b", "scene-1"]]),
    );
    expect(annotationInScene(split, "scene-1b").shape).toEqual(
      text(400, "Hello"),
    );
    const [gone] = withSceneShapesFollowing(clips, []);
    expect(gone.sceneShapes).toBeUndefined();
    expect(withoutSceneShape(placed, "scene-1").sceneShapes).toBeUndefined();
  });
});
