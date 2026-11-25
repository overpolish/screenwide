// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultScreenshotOutput } from "../screenshot-output";

import {
  withScreenshotMainBackground,
  withScreenshotShapeFollowed,
} from "./screenshot-format-sharing";
import {
  addedScreenshotFormat,
  switchedScreenshotFormat,
} from "./screenshot-formats";

import type { ScreenshotWorkspaceOutputSettings } from "../screenshot-output";

const placed = {
  cropHeight: 800,
  cropWidth: 1200,
  cropX: 360,
  cropY: 140,
  imageWidth: 1200,
  imageX: 360,
  imageY: 140,
};

const main = (): ScreenshotWorkspaceOutputSettings => ({
  ...defaultScreenshotOutput(1920, 1080),
  ...placed,
  backgroundColor: "#112233",
  items: [
    { id: 1, output: { ...defaultScreenshotOutput(1920, 1080), ...placed } },
  ],
});

const blank = new Set<number>();

/** The main canvas with TikTok added and put in hand. */
const withTikTok = () => {
  const added = addedScreenshotFormat(
    null,
    { blank, workspace: main() },
    "tiktok",
  );
  if (!added) throw new Error("TikTok is a template");
  return added;
};

const backToMain = (
  added: ReturnType<typeof withTikTok>,
  workspace: ScreenshotWorkspaceOutputSettings,
) => {
  const back = switchedScreenshotFormat(
    added.formats,
    { blank, workspace },
    added.formats.list[0].id,
  );
  if (!back) throw new Error("The main canvas can be put in hand");
  return back;
};

describe("a screenshot's platforms", () => {
  it("lays the main canvas out on the platform's canvas", () => {
    const { document } = withTikTok();
    expect([document.width, document.height]).toEqual([1080, 1920]);
    const [layer] = document.items;
    expect(layer.output.cropX).toBeGreaterThanOrEqual(0);
    expect(layer.output.cropX + layer.output.cropWidth).toBeLessThanOrEqual(
      1080,
    );
  });

  it("keeps a background changed on a platform there, until it follows the main canvas again", () => {
    const added = withTikTok();
    const back = backToMain(added, {
      ...added.document,
      backgroundColor: "#ff0000",
    });
    expect(back.document.backgroundColor).toBe("#112233");
    expect(back.document.width).toBe(1920);
    const again = switchedScreenshotFormat(
      back.formats,
      { blank, workspace: back.document },
      "tiktok",
    );
    expect(again?.document.backgroundColor).toBe("#ff0000");
    if (!again) throw new Error("TikTok can be put in hand");
    const reset = withScreenshotMainBackground(again.formats, {
      blank,
      workspace: again.document,
    });
    expect(reset?.document.backgroundColor).toBe("#112233");
    expect(reset?.formats.list[1].ownBackground).toBeUndefined();
  });

  it("keeps a layer moved on a platform there, and the main canvas as it was", () => {
    const added = withTikTok();
    const [layer] = added.document.items;
    const moved = {
      ...added.document,
      items: [
        {
          ...layer,
          output: { ...layer.output, cropY: layer.output.cropY + 300 },
        },
      ],
    };
    const back = backToMain(added, moved);
    expect(back.document.items[0].output.cropY).toBe(placed.cropY);
    expect(back.formats.list[1].placed).toBe(true);

    const again = switchedScreenshotFormat(
      back.formats,
      { blank, workspace: back.document },
      "tiktok",
    );
    expect(again?.document.items[0].output.cropY).toBe(
      layer.output.cropY + 300,
    );
  });

  it("lets a following platform take up another's framing, and keeps it there", () => {
    const added = withTikTok();
    const [layer] = added.document.items;
    const turned = {
      ...added.document,
      items: [
        {
          ...layer,
          output: { ...layer.output, cropY: layer.output.cropY + 200 },
        },
      ],
    };
    // Shorts follows the main canvas, then takes up TikTok's framing.
    const shorts = addedScreenshotFormat(
      added.formats,
      { blank, workspace: turned },
      "youtube-shorts",
    );
    if (!shorts) throw new Error("Shorts is a template");
    const tikTokShape = shorts.formats.list[1].id;
    const shared = withScreenshotShapeFollowed(
      shorts.formats,
      { blank, workspace: shorts.document },
      tikTokShape,
    );
    expect(shared?.formats.list).toHaveLength(2);
    expect(shared?.formats.list[1].platforms).toEqual([
      "tiktok",
      "youtube-shorts",
    ]);
    expect(shared?.document.items[0].output.cropY).toBe(
      layer.output.cropY + 200,
    );
  });
});
