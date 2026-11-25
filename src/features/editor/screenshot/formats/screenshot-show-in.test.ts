// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  drawnWorkspace,
  layerSyncId,
  leftOutInHand,
  screenshotFormatSync,
  withLeftOutKept,
  withScreenshotShownToggled,
} from "./screenshot-show-in";

import type { ScreenshotFormats } from "../../../../bindings/ScreenshotFormats";

const formats = (active: string): ScreenshotFormats => ({
  active,
  list: [
    { id: "main" },
    { id: "tall", platforms: ["tiktok", "youtube-shorts"] },
  ],
});

describe("a screenshot's show in", () => {
  it("leaves a layer out of one platform and nowhere else", () => {
    const left = withScreenshotShownToggled(formats("tall"), {
      format: "tiktok",
      syncId: layerSyncId(7),
    });
    expect(
      screenshotFormatSync(left, {
        key: null,
        syncId: layerSyncId(7),
      })?.formats.map((format) => format.isShown),
    ).toEqual([true, false, true]);
    expect([...leftOutInHand(left).layers]).toEqual([7]);
    expect(leftOutInHand({ ...left, active: "main" }).layers.size).toBe(0);
  });

  it("keeps an annotation shown in the one format it shows in", () => {
    let only = formats("main");
    for (const format of ["tiktok", "youtube-shorts"])
      only = withScreenshotShownToggled(only, { format, syncId: "a" });
    expect(
      withScreenshotShownToggled(only, { format: "main", syncId: "a" }),
    ).toBe(only);
  });

  it("keeps what a format leaves out through an edit made there", () => {
    const leftOut = { annotations: new Set(["b"]), layers: new Set<number>() };
    const annotations = ["a", "b", "c"].map((id) => ({ id }));
    const workspace = {
      items: [{ id: 1, output: { annotations } }],
    } as unknown as Parameters<typeof drawnWorkspace>[0];
    const drawn = drawnWorkspace(workspace, leftOut);
    const shown = drawn.items[0].output.annotations.map(({ id }) => ({ id }));
    expect(shown.map(({ id }) => id)).toEqual(["a", "c"]);
    // "c" deleted there: "b" comes back where it was.
    const kept = withLeftOutKept(shown.slice(0, 1), {
      leftOut: leftOut.annotations,
      previous: annotations,
    });
    expect(kept.map(({ id }) => id)).toEqual(["a", "b"]);
  });
});
