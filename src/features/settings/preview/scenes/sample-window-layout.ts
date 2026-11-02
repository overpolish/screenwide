// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Box, FONT, type Spot, TYPE_SIZE } from "./scene-kit";

// Where an app window's parts go on the sample desktop, worked out apart
// from the painting (`sample-window.ts`), so a scene's pointer goes where
// its button is.

export type AppWindow = {
  box: Box;
  button?: string;
  chart?: boolean;
  /** What has been typed in the window's field. */
  field?: string;
  heading?: string;
  /** Lines of text too small to read, each a share of the content's width. */
  lines?: readonly number[];
  /** Lines of text to read, such as OCR's, the first in bold. */
  paragraphs?: readonly string[];
  sidebar?: boolean;
};

/** The title bar, as tall as the platform's. */
export const CHROME = 28;
const PAD = 14;
const HEADING_SIZE = 15;
const FIELD_HEIGHT = 26;
const BUTTON_HEIGHT = 24;
const LINE_STEP = 13;
export const PARAGRAPH_LEADING = 20;
export const SIDEBAR_ROWS = 5;

let measuring: CanvasRenderingContext2D | null = null;

/** How wide `text` sets in `font`. */
export function textWidth(text: string, font: string) {
  measuring ??= document.createElement("canvas").getContext("2d");
  if (!measuring) return 0;
  measuring.font = font;
  return measuring.measureText(text).width;
}

export const buttonFont = `600 ${String(TYPE_SIZE - 1)}px ${FONT}`;
export const headingFont = `600 ${String(HEADING_SIZE)}px ${FONT}`;
export const textFont = (weight: number) =>
  `${String(weight)} ${String(TYPE_SIZE)}px ${FONT}`;

/** Where each of the window's parts goes. */
export function windowParts(window: AppWindow) {
  const { box } = window;
  const sidebarWidth = window.sidebar
    ? Math.round(Math.min(84, box.width * 0.26))
    : 0;
  const content: Box = {
    height: box.height - CHROME - PAD * 2,
    width: box.width - sidebarWidth - PAD * 2,
    x: box.x + sidebarWidth + PAD,
    y: box.y + CHROME + PAD - 2,
  };
  const bottom = content.y + content.height;
  let y = content.y;
  const heading: Spot | null = window.heading
    ? [content.x, y + HEADING_SIZE - 2]
    : null;
  if (heading) y += HEADING_SIZE + 12;
  const field: Box | null =
    window.field === undefined
      ? null
      : { height: FIELD_HEIGHT, width: content.width, x: content.x, y };
  if (field) y += FIELD_HEIGHT + 12;
  const paragraphs = (window.paragraphs ?? []).map((_, index): Spot => [
    content.x,
    y + TYPE_SIZE - 2 + index * PARAGRAPH_LEADING,
  ]);
  y += paragraphs.length * PARAGRAPH_LEADING;
  const lines = (window.lines ?? []).map((share, index): Box => ({
    height: 6,
    width: Math.round(content.width * share),
    x: content.x,
    y: y + index * LINE_STEP,
  }));
  if (lines.length > 0) y += lines.length * LINE_STEP + 6;
  const buttonWidth = window.button
    ? Math.round(textWidth(window.button, buttonFont) + 28)
    : 0;
  const button: Box | null = window.button
    ? {
        height: BUTTON_HEIGHT,
        width: buttonWidth,
        x: content.x + content.width - buttonWidth,
        y: bottom - BUTTON_HEIGHT,
      }
    : null;
  const chartBottom = button ? button.y - 12 : bottom;
  const chart: Box | null =
    window.chart && chartBottom - y >= 28
      ? { height: chartBottom - y, width: content.width, x: content.x, y }
      : null;
  return {
    button,
    chart,
    content,
    field,
    heading,
    lines,
    paragraphs,
    sidebarWidth,
  };
}
