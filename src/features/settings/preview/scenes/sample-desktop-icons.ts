// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { isWindows } from "../../../../lib/platform";

import { SAMPLE } from "./sample-palette";
import { menuBarHeight } from "./sample-system-bars";
import { FONT, type Painter } from "./scene-kit";

// A few icons on the sample desktop, lined up where the system puts them:
// down the screen's end under the menu bar on macOS, down its start on
// Windows, in the room the sample windows leave at either side. Labels are
// white with a soft shadow, as both systems draw them over a picture.

const ICON = 32;
/** From one icon's top to the next. */
const STEP = 62;
/** From the screen's edge to the column's middle. */
const INSET = 32;
const LABEL_SIZE = 10;

type DesktopIcon = { kind: "file" | "folder"; label: string };

const icons = (): DesktopIcon[] => [
  { kind: "folder", label: t("settings-preview-desktop-projects") },
  { kind: "folder", label: t("settings-preview-desktop-screenshots") },
  { kind: "file", label: t("settings-preview-desktop-notes") },
];

function paintFolder(context: CanvasRenderingContext2D, x: number, y: number) {
  context.fillStyle = SAMPLE.folderTab;
  context.beginPath();
  context.roundRect(x + 2, y + 4, ICON * 0.42, 8, 2);
  context.fill();
  context.fillStyle = SAMPLE.folder;
  context.beginPath();
  context.roundRect(x, y + 8, ICON, ICON - 10, 3);
  context.fill();
}

function paintFile(context: CanvasRenderingContext2D, x: number, y: number) {
  const left = x + 5;
  const width = ICON - 10;
  context.fillStyle = SAMPLE.window;
  context.beginPath();
  context.roundRect(left, y, width, ICON, 2);
  context.fill();
  context.fillStyle = SAMPLE.line;
  for (let line = 0; line < 4; line++) {
    context.fillRect(left + 4, y + 9 + line * 5, width - 8, 2);
  }
}

export const paintDesktopIcons: Painter = (context, size, { notch }) => {
  const middle = isWindows() ? INSET : size.width - INSET;
  let top = menuBarHeight(notch) + 16;
  context.font = `500 ${String(LABEL_SIZE)}px ${FONT}`;
  context.textAlign = "center";
  for (const icon of icons()) {
    const x = middle - ICON / 2;
    context.save();
    context.shadowColor = SAMPLE.desktopShadow;
    context.shadowBlur = 4;
    context.shadowOffsetY = 1;
    if (icon.kind === "folder") paintFolder(context, x, top);
    else paintFile(context, x, top);
    context.fillStyle = SAMPLE.desktopInk;
    context.fillText(icon.label, middle, top + ICON + 6 + LABEL_SIZE);
    context.restore();
    top += STEP;
  }
  context.textAlign = "start";
};
