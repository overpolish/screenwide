// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { insetScreenshotPadding } from "./screenshot-inset";
import {
  ScreenshotWorkspaceOutputSettings,
  defaultScreenshotOutput,
  resetScreenshotLayout,
  screenshotLayout,
  screenshotWorkspaceItemOutput,
} from "./screenshot-output";
import {
  normalizedScreenshotSelection,
  screenshotSelectionTargets,
} from "./screenshot-selection";

describe("a padded screenshot layer", () => {
  it("hands the hit target the padded box the selection is drawn on", () => {
    const item = { height: 500, id: 1, scaleFactor: 2, width: 1_000 };
    const canvas = defaultScreenshotOutput(1_600, 900);
    const padded = insetScreenshotPadding(
      resetScreenshotLayout(canvas, item),
      item,
      6,
    );
    const workspace: ScreenshotWorkspaceOutputSettings = {
      ...canvas,
      items: [{ id: item.id, output: padded }],
    };
    const dimensions = { height: 900, width: 1_600 };
    const [target] = screenshotSelectionTargets([item], workspace, {
      ...dimensions,
      cropMode: false,
    });
    // What the editor sends as the selection of the layer in hand.
    const selection = normalizedScreenshotSelection(
      screenshotLayout(item, screenshotWorkspaceItemOutput(workspace, item.id)),
      dimensions,
      "select",
    );
    expect(target.rect).toEqual(selection.rect);
    const { crop, sourceCrop } = screenshotLayout(item, padded);
    // 6% of the 900 pixel high canvas on each side.
    expect(crop.width - sourceCrop.width).toBeCloseTo(2 * 54);
    expect(target.rect.width).toBeCloseTo(crop.width / dimensions.width);
    expect(target.rect.x).toBeCloseTo(crop.x / dimensions.width);
  });
});
