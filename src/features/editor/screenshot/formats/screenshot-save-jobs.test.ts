// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultScreenshotOutput } from "../screenshot-output";

import { screenshotSaveJobs } from "./screenshot-save-jobs";
import { layerSyncId, withScreenshotShownToggled } from "./screenshot-show-in";

import type { ScreenshotFormats } from "../../../../bindings/ScreenshotFormats";

const layer = (id: number, cropX: number) => ({
  id,
  output: {
    ...defaultScreenshotOutput(1920, 1080),
    cropHeight: 400,
    cropWidth: 600,
    cropX,
    cropY: 300,
    imageWidth: 600,
    imageX: cropX,
    imageY: 300,
  },
});

const inHand = {
  blank: new Set<number>(),
  workspace: {
    ...defaultScreenshotOutput(1920, 1080),
    items: [layer(1, 100), layer(2, 1100)],
  },
};

const formats: ScreenshotFormats = {
  active: "main",
  list: [
    { id: "main" },
    { id: "tall", platforms: ["tiktok", "youtube-shorts"] },
  ],
};

describe("exporting every format of a screenshot", () => {
  it("names each format and writes a platform sharing another's framing as a copy", () => {
    const jobs = screenshotSaveJobs({ fileStem: "Shot", formats, inHand });
    expect(jobs?.map(({ fileStem, isCopy }) => [fileStem, isCopy])).toEqual([
      ["Shot 1,920 × 1,080", false],
      ["Shot TikTok", false],
      ["Shot YouTube Shorts", true],
    ]);
    expect(jobs?.[1].workspace).toMatchObject({ height: 1920, width: 1080 });
  });

  it("writes a platform leaving out a layer its own way, without it", () => {
    const apart = withScreenshotShownToggled(formats, {
      format: "youtube-shorts",
      syncId: layerSyncId(2),
    });
    const jobs = screenshotSaveJobs({
      fileStem: "Shot",
      formats: apart,
      inHand,
    });
    const shorts = jobs?.find((job) => job.fileStem === "Shot YouTube Shorts");
    expect(shorts?.isCopy).toBe(false);
    expect(shorts?.workspace.items.map((item) => item.id)).toEqual([1]);
  });
});
