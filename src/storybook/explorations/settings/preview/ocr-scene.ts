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

// OCR: a note and a QR code. An area is dragged over both and read, then
// two of the note's lines are selected.

const NOTE_LINES = [
  "Launch checklist",
  "Record the demo at 60 fps",
  "Ship on Thursday",
];
const NOTE_LEADING = 22;

// https://github.com/overpolish/screenwide at error correction M, 29
// modules square.
const QR_MODULES = [
  "11111110000100101001101111111",
  "10000010000001110110101000001",
  "10111010110110101010101011101",
  "10111010110101100100001011101",
  "10111010110111001001101011101",
  "10000010111110011010001000001",
  "11111110101010101010101111111",
  "00000000100010111011100000000",
  "10111110001011110100101111100",
  "10101101100010001111101110001",
  "11111011011000111100100010000",
  "10110001001000011001100001010",
  "11010111110101110101000001100",
  "01110000101010001111111110001",
  "10101111001100110000110011100",
  "00011100011110100011111110010",
  "00110010010010011101010101100",
  "11011101101011001111011110101",
  "10111111100001111010101110100",
  "10001000000001000000111000010",
  "10011011001000110101111110111",
  "00000000111000001000100011111",
  "11111110000100011111101011100",
  "10000010110010100001100010010",
  "10111010101000011100111110111",
  "10111010101110101111100001111",
  "10111010110001011011111111110",
  "10000010000011101010110101010",
  "11111110101110100111001110100",
];
const QR_MODULE_SIZE = 2.25;

function ocrLayout(width: number) {
  const note: Box = {
    height: 90,
    width: 210,
    x: Math.round(width / 2 - 170),
    y: 16,
  };
  const code: Box = {
    height: 84,
    width: 84,
    x: note.x + note.width + 22,
    y: 19,
  };
  const text = { x: note.x + 14, y: note.y + 28 };
  return { code, note, text };
}

let measure: CanvasRenderingContext2D | null = null;

function lineWidth(line: string) {
  measure ??= document.createElement("canvas").getContext("2d");
  if (!measure) return 0;
  measure.font = `${String(TYPE_SIZE)}px ${FONT}`;
  return measure.measureText(line).width;
}

export const OCR_SCENE: ToolScene = {
  duration: 9,
  paint(context, width, height) {
    paintWallpaper(context, width, height);
    const { code, note, text } = ocrLayout(width);
    paintCard(context, note, 8);
    context.fillStyle = INK;
    context.textBaseline = "alphabetic";
    NOTE_LINES.forEach((line, index) => {
      const weight = index === 0 ? 600 : 400;
      context.font = `${String(weight)} ${String(TYPE_SIZE)}px ${FONT}`;
      context.fillText(line, text.x, text.y + index * NOTE_LEADING);
    });
    paintCard(context, code, 6);
    const inset = (code.width - QR_MODULES.length * QR_MODULE_SIZE) / 2;
    context.fillStyle = "#000";
    QR_MODULES.forEach((row, y) => {
      for (let x = 0; x < row.length; x += 1) {
        if (row[x] !== "1") continue;
        context.fillRect(
          code.x + inset + x * QR_MODULE_SIZE,
          code.y + inset + y * QR_MODULE_SIZE,
          QR_MODULE_SIZE,
          QR_MODULE_SIZE,
        );
      }
    });
  },
  pointer(at, width) {
    const { code, note, text } = ocrLayout(width);
    const corner: Spot = [note.x - 10, note.y - 8];
    const across: Spot = [code.x + code.width + 10, note.y + note.height + 8];
    // The middle of a line's x-height, where a press lands on its text.
    const lineY = (index: number) =>
      text.y + index * NOTE_LEADING - TYPE_SIZE * 0.35;
    const from: Spot = [text.x + 1, lineY(1)];
    const to: Spot = [text.x + lineWidth(NOTE_LINES[2]) - 1, lineY(2)];
    if (at < 0.5) {
      return { pressed: false, x: corner[0], y: corner[1] };
    }
    if (at < 1.8) {
      return { pressed: true, ...travel(at, [0.5, 1.7], [corner, across]) };
    }
    if (at < 3.4) {
      return { pressed: false, ...travel(at, [2.4, 3.3], [across, from]) };
    }
    if (at < 4.8) {
      return { pressed: true, ...travel(at, [3.5, 4.6], [from, to]) };
    }
    if (at < 8.4) {
      return { pressed: false, x: to[0], y: to[1] };
    }
    return null;
  },
};
