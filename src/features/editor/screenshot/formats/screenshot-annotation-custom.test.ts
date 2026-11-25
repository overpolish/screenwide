// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultScreenshotOutput } from "../screenshot-output";

import { withScreenshotCustomToggled } from "./screenshot-annotation-custom";
import {
  addedScreenshotFormat,
  switchedScreenshotFormat,
} from "./screenshot-formats";

import type { ScreenshotWorkspaceOutputSettings } from "../screenshot-output";

type Workspace = ScreenshotWorkspaceOutputSettings;
type Annotation = Workspace["items"][number]["output"]["annotations"][number];

const arrow = (x: number) =>
  ({ id: "arrow", shape: { x }, style: {} }) as unknown as Annotation;

const placed = {
  cropHeight: 800,
  cropWidth: 1200,
  cropX: 360,
  cropY: 140,
  imageWidth: 1200,
  imageX: 360,
  imageY: 140,
};

const main = (): Workspace => ({
  ...defaultScreenshotOutput(1920, 1080),
  ...placed,
  items: [
    {
      id: 1,
      output: {
        ...defaultScreenshotOutput(1920, 1080),
        ...placed,
        annotations: [arrow(10)],
      },
    },
  ],
});

const blank = new Set<number>();

const withArrowAt = (workspace: Workspace, x: number): Workspace => ({
  ...workspace,
  items: workspace.items.map((item) => ({
    ...item,
    output: { ...item.output, annotations: [arrow(x)] },
  })),
});

const arrowOf = (workspace: Workspace | undefined) =>
  workspace?.items[0].output.annotations[0];

describe("a screenshot's annotation on a platform", () => {
  it("stays custom on its proportion when changed there, and follows the main canvas again when unticked", () => {
    const added = addedScreenshotFormat(
      null,
      { blank, workspace: main() },
      "tiktok",
    );
    if (!added) throw new Error("TikTok is a template");
    const moved = withArrowAt(added.document, 99);
    const back = switchedScreenshotFormat(
      added.formats,
      { blank, workspace: moved },
      added.formats.list[0].id,
    );
    expect(arrowOf(back?.document)).toEqual(arrow(10));
    if (!back) throw new Error("The main canvas can be put in hand");
    // Instagram's story is 9:16 too, so it shows TikTok's placement.
    const story = addedScreenshotFormat(
      back.formats,
      { blank, workspace: back.document },
      "instagram-story",
    );
    expect(arrowOf(story?.document)).toEqual(arrow(99));
    if (!story) throw new Error("A story is a template");
    const followed = withScreenshotCustomToggled(
      story.formats,
      { blank, workspace: story.document },
      "arrow",
    );
    expect(arrowOf(followed?.document)).toEqual(arrow(10));
    expect(followed?.formats.layouts?.arrow).toBeUndefined();
  });
});
