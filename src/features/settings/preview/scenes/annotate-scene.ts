// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";

import { paintAppWindow } from "./sample-window";
import { type AppWindow, windowParts } from "./sample-window-layout";
import {
  type Box,
  paintWallpaper,
  type Spot,
  TOOL_SCALE,
  type ToolScene,
  travel,
} from "./scene-kit";

// Annotate: an app's window and its button. An arrow is drawn to the
// window's heading and a counter dropped to mark its button; then the
// session ends, taking them with it unless they are kept, and a kept pass
// fades as the next begins.

export function annotateLayout(width: number): AppWindow {
  return {
    box: { height: 126, width: 286, x: Math.round(width / 2 - 90), y: 18 },
    button: t("settings-preview-window-button"),
    heading: t("settings-preview-window-title"),
    // The second line stops short, leaving room for the largest counter
    // beside the button.
    lines: [0.86, 0.5],
  };
}

/** How far a counter's tail reaches from its disc's centre, in radii: the
 * twin of `COUNTER_TAIL_REACH` in the annotations crate. */
const TAIL_REACH = 1.5;

/** Where the tail of a counter marking `button` touches it: on the
 * button's edge, on the side the tail comes from, so the disc sits
 * outside the button and its label stays readable. `angle` is where the
 * tail points, in radians clockwise from east. */
function counterTip(button: Box, angle: number): Spot {
  const [x, y] = [Math.cos(angle), Math.sin(angle)];
  const half: Spot = [button.width / 2, button.height / 2];
  const reach = Math.min(
    x === 0 ? Infinity : half[0] / Math.abs(x),
    y === 0 ? Infinity : half[1] / Math.abs(y),
  );
  return [button.x + half[0] - x * reach, button.y + half[1] - y * reach];
}

/** Where a counter `size` points across, its tail at `angle`, is dropped
 * for its tail's tip to land on `tip`, in the picture's points. The tool
 * drops a counter by its centre, so the pointer clicks there. */
function counterCentre(tip: Spot, size: number, angle: number): Spot {
  const reach = (size / 2) * TOOL_SCALE * TAIL_REACH;
  return [tip[0] - Math.cos(angle) * reach, tip[1] - Math.sin(angle) * reach];
}

export const ANNOTATE_SCENE: ToolScene = {
  cues: [
    { at: 0.1, input: "pickArrow" },
    { at: 1.4, input: "pickCounter" },
    { at: 3.4, input: "exit" },
    { at: 4.4, input: "enter" },
  ],
  duration: 6.4,
  glass: true,
  kind: "tool",
  paint(context, size, { wallpaper }) {
    paintWallpaper(context, size, wallpaper);
    paintAppWindow(context, annotateLayout(size.width));
  },
  pointer(at, { width }, scene) {
    if (scene.kind !== "annotate") return null;
    const window = annotateLayout(width);
    const { button, heading } = windowParts(window);
    if (!button || !heading) return null;
    const tail: Spot = [window.box.x - 110, window.box.y + 14];
    const head: Spot = [heading[0] - 6, heading[1] - 5];
    const counter = counterCentre(
      counterTip(button, scene.counterAngle),
      scene.counterSize,
      scene.counterAngle,
    );
    if (at < 0.3) {
      return { pressed: false, x: tail[0], y: tail[1] };
    }
    if (at < 1.2) {
      return { pressed: true, ...travel(at, [0.35, 1.1], [tail, head]) };
    }
    if (at < 1.6) {
      return { pressed: false, ...travel(at, [1.2, 1.55], [head, counter]) };
    }
    if (at < 1.75) {
      return { pressed: true, x: counter[0], y: counter[1] };
    }
    return null;
  },
};
