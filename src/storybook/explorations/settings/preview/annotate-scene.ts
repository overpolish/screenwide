// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  type Box,
  FONT,
  INK,
  paintCard,
  paintWallpaper,
  type Spot,
  type ToolScene,
  travel,
  TYPE_SIZE,
} from "./scene-kit";

// Annotate: a window with a button. An arrow is drawn to the button and a
// counter dropped beside it; then the session ends and the next starts,
// bringing them back only if they are kept.

function annotateLayout(width: number) {
  const window: Box = {
    height: 112,
    width: 260,
    x: Math.round(width / 2 - 100),
    y: 24,
  };
  const button: Box = {
    height: 26,
    width: 76,
    x: window.x + window.width - 92,
    y: window.y + window.height - 42,
  };
  return { button, window };
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
  paint(context, width, height) {
    paintWallpaper(context, width, height);
    const { button, window } = annotateLayout(width);
    paintCard(context, window, 10);
    context.fillStyle = INK;
    context.font = `600 ${String(TYPE_SIZE)}px ${FONT}`;
    context.fillText("Publish changes", window.x + 16, window.y + 30);
    context.beginPath();
    context.roundRect(button.x, button.y, button.width, button.height, 6);
    context.fill();
    context.fillStyle = "#fff";
    context.font = `500 ${String(TYPE_SIZE - 1)}px ${FONT}`;
    context.textAlign = "center";
    context.fillText(
      "Publish",
      button.x + button.width / 2,
      button.y + button.height / 2 + 4,
    );
    context.textAlign = "start";
  },
  pointer(at, width) {
    const { button, window } = annotateLayout(width);
    const tail: Spot = [window.x - 80, window.y + 10];
    const head: Spot = [button.x - 8, button.y + button.height / 2];
    const counter: Spot = [button.x + button.width + 28, button.y - 18];
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
