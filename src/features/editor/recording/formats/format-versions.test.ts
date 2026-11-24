// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { formatVersions } from "./format-versions";

import type { FormatContext } from "./format-scope";
import type { FormatEdits } from "../../../../bindings/FormatEdits";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

const edit = {
  annotationClips: [
    { annotation: { id: "arrow" }, endMs: 2_000, startMs: 1_000 },
    { annotation: { id: "circle" }, endMs: 6_000, startMs: 5_000 },
  ],
  artifactId: 1,
  sceneClips: [
    { endMs: 9_000, id: "scene", preset: "stacked", startMs: 7_000 },
  ],
} as unknown as RecordingTimelineEdit;

const shapes = [
  { annotationKey: "wide", formats: ["youtube-1080"], id: "wide" },
  {
    annotationKey: "tall",
    formats: ["tiktok", "youtube-shorts", "linkedin-vertical"],
    id: "tall",
  },
];
const formats = shapes.flatMap((shape) => shape.formats);
const contextOf = (id: string): FormatContext => {
  const shape = id === "youtube-1080" ? "wide" : "tall";
  return { annotationKey: shape, id, shape, shapes };
};
const versions = (edits: FormatEdits) =>
  formatVersions({ contextOf, edit, edits, formats }).map((version) => ({
    formats: version.formats,
    marks: version.marks.map((mark) => mark.id),
  }));

describe("the versions a recording comes out in", () => {
  it("is none while every format comes out the same", () => {
    expect(versions({})).toEqual([]);
  });

  it("puts formats that include the same things on one line", () => {
    expect(
      versions({
        annotations: {
          arrow: { showIn: { formats: ["youtube-1080", "youtube-shorts"] } },
        },
      }),
    ).toEqual([
      { formats: ["youtube-1080", "youtube-shorts"], marks: ["arrow"] },
      { formats: ["tiktok", "linkedin-vertical"], marks: [] },
    ]);
  });

  it("marks a layout of a shape's own on that shape's line alone", () => {
    expect(
      versions({
        scenes: { scene: { layouts: { tall: { preset: "full" } } } },
      }),
    ).toEqual([
      { formats: ["youtube-1080"], marks: [] },
      {
        formats: ["tiktok", "youtube-shorts", "linkedin-vertical"],
        marks: ["scene"],
      },
    ]);
  });
});
