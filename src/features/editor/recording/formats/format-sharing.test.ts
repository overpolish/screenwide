// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultCameraOverlay } from "../../export/recording-export-settings";
import { defaultRecordingOutput } from "../../screenshot/screenshot-output";

import {
  removedPlatform,
  withMainPlacement,
  withPlacedHere,
  withPlatformShared,
  withShapeFollowed,
} from "./format-sharing";
import { addedFormat, switchedFormat } from "./recording-formats";

import type { FormatDocument } from "./format-document";

const vertical = { height: 1_920, width: 1_080 };

const main: FormatDocument = {
  cameraOverlay: defaultCameraOverlay(),
  recordingOutput: defaultRecordingOutput({
    primary: { height: 1_080, width: 1_920 },
  }),
};

/** TikTok and Shorts, each in a shape of its own, Shorts in hand. */
const both = () => {
  const tiktok = addedFormat(null, main, { preset: "tiktok", size: vertical });
  return addedFormat(tiktok.formats, tiktok.document, {
    preset: "youtube-shorts",
    size: vertical,
  });
};

/** TikTok in hand with its screen moved there. */
const tiktokPlaced = () => {
  const added = both();
  const inHand = switchedFormat(added.formats, added.document, "tiktok");
  if (!inHand) throw new Error("TikTok is a format");
  const moved: FormatDocument = {
    ...inHand.document,
    recordingOutput: {
      ...inHand.document.recordingOutput,
      primary: { ...inHand.document.recordingOutput.primary, cropX: 10 },
    },
  };
  const formats = withPlacedHere(inHand.formats) ?? inHand.formats;
  return { document: moved, formats };
};

describe("format sharing", () => {
  it("shares a platform's own framing with another of its canvas", () => {
    const { document, formats } = tiktokPlaced();
    const shared = withPlatformShared(formats, document, {
      isShared: true,
      platform: "youtube-shorts",
    });
    expect(shared?.formats.list).toHaveLength(2);
    expect(shared?.formats.list[1]?.platforms).toEqual([
      "tiktok",
      "youtube-shorts",
    ]);
    // Placed its own way, the screen stays where it was moved, kept clear
    // of what both apps cover.
    expect(shared?.document.recordingOutput.primary.cropX).toBe(10);
    expect(shared?.document.recordingOutput.primary.clearArea).toEqual({
      bottom: 0.25,
      side: 0.05,
      top: 0.09,
    });
  });

  it("gives a platform taken out back to following the main canvas", () => {
    const { document, formats } = tiktokPlaced();
    const shared = withPlatformShared(formats, document, {
      isShared: true,
      platform: "youtube-shorts",
    });
    const apart = shared
      ? withPlatformShared(shared.formats, shared.document, {
          isShared: false,
          platform: "youtube-shorts",
        })
      : null;
    const shorts = apart?.formats.list[2];
    expect(shorts?.platforms).toEqual(["youtube-shorts"]);
    expect(shorts?.placed).toBeUndefined();
    expect(shorts?.look?.recordingOutput.primary.cropX).not.toBe(10);
  });

  it("lets a platform following the main canvas take up another's framing", () => {
    const { document, formats } = tiktokPlaced();
    const shorts = switchedFormat(formats, document, "youtube-shorts");
    const tiktokShape = formats.list[1]?.id ?? "";
    const followed = shorts
      ? withShapeFollowed(shorts.formats, shorts.document, tiktokShape)
      : null;
    expect(followed?.formats.active).toBe(tiktokShape);
    expect(followed?.formats.list).toHaveLength(2);
    expect(followed?.formats.list[1]?.platform).toBe("youtube-shorts");
    expect(followed?.document.recordingOutput.primary.cropX).toBe(10);
  });

  it("lays a placed screen out from the main canvas again", () => {
    const { document, formats } = tiktokPlaced();
    const reset = withMainPlacement(formats, document);
    expect(reset?.formats.list[1]?.placed).toBeUndefined();
    expect(reset?.document.recordingOutput.primary.cropX).not.toBe(10);
  });

  it("keeps a shared shape when one of its platforms is removed", () => {
    const { document, formats } = tiktokPlaced();
    const shared = withPlatformShared(formats, document, {
      isShared: true,
      platform: "youtube-shorts",
    });
    const removed = shared
      ? removedPlatform(shared.formats, shared.document, "youtube-shorts")
      : null;
    expect(removed?.formats?.list[1]?.platforms).toEqual(["tiktok"]);
    expect(removed?.document.recordingOutput.primary.clearArea?.top).toBe(0.07);
  });
});
