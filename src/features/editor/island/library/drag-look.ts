// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { Sound } from "../../sound-track/sound-library";

// A sound dragged out of the library is drawn under the pointer as a chip of
// the island's black holding its waveform and title, the row it came from in
// small, rather than as a file's icon. Drawn at twice its size, so it is
// sharp on a Retina screen.

const WIDTH = 220;
const HEIGHT = 36;
const SCALE = 2;
const BARS = 24;
const WAVE_WIDTH = 64;
const INSET = 14;

/** `sound` drawn as its chip, as PNG bytes and its size in points. */
export function soundDragLook(sound: Sound) {
  const canvas = document.createElement("canvas");
  canvas.width = WIDTH * SCALE;
  canvas.height = HEIGHT * SCALE;
  const context = canvas.getContext("2d");
  if (!context) return null;
  context.scale(SCALE, SCALE);
  const style = getComputedStyle(document.documentElement);
  context.fillStyle = "#000";
  context.beginPath();
  context.roundRect(0, 0, WIDTH, HEIGHT, HEIGHT / 2);
  context.fill();
  context.fillStyle = "#fff";
  const middle = HEIGHT / 2;
  const levels = sound.levels;
  for (let bar = 0; bar < BARS; bar += 1) {
    const peak =
      levels.length === 0
        ? 0
        : Math.max(
            ...levels.slice(
              Math.floor((bar / BARS) * levels.length),
              Math.max(
                Math.floor((bar / BARS) * levels.length) + 1,
                Math.floor(((bar + 1) / BARS) * levels.length),
              ),
            ),
          );
    const reach = Math.max(1, Math.pow(Math.min(peak, 1), 0.55) * (middle - 9));
    const x = INSET + (bar + 0.5) * (WAVE_WIDTH / BARS);
    context.fillRect(x - 0.75, middle - reach, 1.5, reach * 2);
  }
  context.font = `500 13px ${style.getPropertyValue("--font-sans") || "system-ui"}`;
  context.textBaseline = "middle";
  const textX = INSET * 2 + WAVE_WIDTH;
  const room = WIDTH - textX - INSET;
  let title = sound.title;
  while (title.length > 1 && context.measureText(title).width > room)
    title = title.slice(0, -1);
  if (title !== sound.title) title = `${title.slice(0, -1)}…`;
  context.fillText(title, textX, middle);
  const data = canvas.toDataURL("image/png").split(",")[1] ?? "";
  const binary = atob(data);
  return {
    height: HEIGHT,
    png: Array.from(binary, (letter) => letter.charCodeAt(0)),
    width: WIDTH,
  };
}
