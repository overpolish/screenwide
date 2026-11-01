// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ToolPanelComparedFrame } from "../../tool-panels/tool-panel-export";

/**
 * Where the exported picture is drawn over the Original one, in the
 * Original's pixels, so the look's canvas lies in the same place in both.
 * An export drawn smaller fits the canvas at one scale into sides rounded
 * to even, a fraction of a pixel cropped or padded; stretching the exported
 * picture to the Original's size instead would drift features apart
 * towards its edges.
 */
export function comparedPlacement(
  original: ToolPanelComparedFrame,
  exported: ToolPanelComparedFrame,
) {
  const scaleX = original.canvas.width / exported.canvas.width;
  const scaleY = original.canvas.height / exported.canvas.height;
  return {
    height: exported.height * scaleY,
    left: original.canvas.x - exported.canvas.x * scaleX,
    top: original.canvas.y - exported.canvas.y * scaleY,
    width: exported.width * scaleX,
  };
}
