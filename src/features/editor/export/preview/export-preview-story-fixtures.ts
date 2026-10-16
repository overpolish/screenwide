// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComparedFrame } from "./export-preview-compare";

/** Sample pictures for the comparison stories: a busy, screen-like frame
 * drawn full size, and the same frame drawn at half size and compressed
 * hard, standing in for an export that is smaller in both. */
export const storyFrames = (): {
  exported: ComparedFrame;
  original: ComparedFrame;
} => {
  const width = 1920;
  const height = 1080;
  const full = document.createElement("canvas");
  full.width = width;
  full.height = height;
  const context = full.getContext("2d");
  if (context) {
    const gradient = context.createLinearGradient(0, 0, width, height);
    gradient.addColorStop(0, "#5b5bd6");
    gradient.addColorStop(1, "#d6409f");
    context.fillStyle = gradient;
    context.fillRect(0, 0, width, height);
    context.fillStyle = "#111318";
    context.fillRect(160, 120, 1600, 840);
    context.font = "22px monospace";
    for (let line = 0; line < 30; line += 1) {
      context.fillStyle = line % 3 === 0 ? "#7ee787" : "#c9d1d9";
      context.fillText(
        `Info Watching C:\\Users\\dom\\coding\\screenwide\\src-tauri\\crates\\export ${line.toString()} for changes`,
        200,
        180 + line * 26,
      );
    }
  }
  const small = document.createElement("canvas");
  small.width = width / 2;
  small.height = height / 2;
  small.getContext("2d")?.drawImage(full, 0, 0, width / 2, height / 2);
  return {
    exported: {
      height: height / 2,
      src: small.toDataURL("image/jpeg", 0.3),
      width: width / 2,
    },
    original: { height, src: full.toDataURL("image/png"), width },
  };
};
