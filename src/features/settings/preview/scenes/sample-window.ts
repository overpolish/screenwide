// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isWindows } from "../../../../lib/platform";

import { SAMPLE } from "./sample-palette";
import {
  type AppWindow,
  buttonFont,
  CHROME,
  headingFont,
  SIDEBAR_ROWS,
  textFont,
  textWidth,
  windowParts,
} from "./sample-window-layout";
import { type Box, TYPE_SIZE } from "./scene-kit";

// An app's window on the sample desktop, painted as apps are: the
// platform's window buttons, a sidebar, a heading, a field with something
// typed in it, text, a chart and the window's primary button, each part
// only where the scene asks for it, where `windowParts` puts it.

function paintFrame(context: CanvasRenderingContext2D, { box }: AppWindow) {
  const radius = isWindows() ? 8 : 10;
  context.save();
  context.shadowColor = SAMPLE.shadow;
  context.shadowBlur = 26;
  context.shadowOffsetY = 10;
  context.fillStyle = SAMPLE.window;
  context.beginPath();
  context.roundRect(box.x, box.y, box.width, box.height, radius);
  context.fill();
  context.restore();
  context.strokeStyle = SAMPLE.hairline;
  context.lineWidth = 1;
  context.stroke();
  context.clip();
}

/** The platform's window buttons: macOS's three lights at the start, or
 * Windows' caption glyphs at the end. */
function paintWindowButtons(
  context: CanvasRenderingContext2D,
  { box }: AppWindow,
) {
  const middle = box.y + CHROME / 2;
  if (!isWindows()) {
    SAMPLE.trafficLights.forEach((colour, index) => {
      context.fillStyle = colour;
      context.beginPath();
      context.arc(box.x + 14 + index * 20, middle, 6, 0, Math.PI * 2);
      context.fill();
    });
    return;
  }
  context.strokeStyle = SAMPLE.caption;
  context.lineWidth = 1;
  const slot = 34;
  const end = box.x + box.width;
  const glyph = 5;
  // Minimise, maximise, close, from the start.
  const close = end - slot / 2;
  const maximise = close - slot;
  const minimise = maximise - slot;
  context.beginPath();
  context.moveTo(minimise - glyph, middle);
  context.lineTo(minimise + glyph, middle);
  context.rect(maximise - glyph, middle - glyph, glyph * 2, glyph * 2);
  context.moveTo(close - glyph, middle - glyph);
  context.lineTo(close + glyph, middle + glyph);
  context.moveTo(close + glyph, middle - glyph);
  context.lineTo(close - glyph, middle + glyph);
  context.stroke();
}

function paintSidebar(
  context: CanvasRenderingContext2D,
  { box }: AppWindow,
  width: number,
) {
  context.fillStyle = SAMPLE.sidebar;
  context.fillRect(box.x, box.y, width, box.height);
  const rows = Array.from({ length: SIDEBAR_ROWS }, (_, index) => index);
  for (const index of rows) {
    const y = box.y + CHROME + 8 + index * 22;
    if (y + 18 > box.y + box.height) break;
    if (index === 0) {
      context.fillStyle = SAMPLE.sidebarSelected;
      context.beginPath();
      context.roundRect(box.x + 6, y, width - 12, 18, 5);
      context.fill();
    }
    context.fillStyle = index === 0 ? SAMPLE.accent : SAMPLE.lineStrong;
    context.beginPath();
    context.roundRect(box.x + 12, y + 4, 10, 10, 3);
    context.fill();
    context.fillStyle = SAMPLE.lineStrong;
    context.beginPath();
    context.roundRect(
      box.x + 28,
      y + 6,
      (width - 40) * (0.9 - index * 0.1),
      6,
      3,
    );
    context.fill();
  }
}

/** Columns of a week's numbers, as the apps' charts draw them. */
const CHART = [0.42, 0.58, 0.5, 0.74, 0.66, 0.88, 0.8];

function paintChart(context: CanvasRenderingContext2D, chart: Box) {
  context.strokeStyle = SAMPLE.line;
  context.lineWidth = 1;
  for (let line = 0; line < 3; line += 1) {
    const y = Math.round(chart.y + (chart.height * line) / 3) + 0.5;
    context.beginPath();
    context.moveTo(chart.x, y);
    context.lineTo(chart.x + chart.width, y);
    context.stroke();
  }
  const step = chart.width / CHART.length;
  const column = Math.min(18, step * 0.55);
  CHART.forEach((share, index) => {
    const height = chart.height * share;
    context.fillStyle =
      index === CHART.length - 2 ? SAMPLE.chart[0] : SAMPLE.chart[1];
    context.beginPath();
    context.roundRect(
      chart.x + step * index + (step - column) / 2,
      chart.y + chart.height - height,
      column,
      height,
      [4, 4, 0, 0],
    );
    context.fill();
  });
}

/** Paints `window` and returns where its parts went. */
export function paintAppWindow(
  context: CanvasRenderingContext2D,
  window: AppWindow,
) {
  const parts = windowParts(window);
  const { box } = window;
  context.save();
  paintFrame(context, window);
  context.fillStyle = SAMPLE.titleBar;
  context.fillRect(box.x, box.y, box.width, CHROME);
  context.fillStyle = SAMPLE.hairline;
  context.fillRect(box.x, box.y + CHROME - 1, box.width, 1);
  if (parts.sidebarWidth > 0) paintSidebar(context, window, parts.sidebarWidth);
  paintWindowButtons(context, window);
  if (parts.heading && window.heading) {
    context.fillStyle = SAMPLE.heading;
    context.font = headingFont;
    context.fillText(window.heading, ...parts.heading);
  }
  if (parts.field && window.field !== undefined) {
    const { field } = parts;
    context.fillStyle = SAMPLE.field;
    context.strokeStyle = SAMPLE.fieldEdge;
    context.beginPath();
    context.roundRect(
      field.x + 0.5,
      field.y + 0.5,
      field.width - 1,
      field.height - 1,
      6,
    );
    context.fill();
    context.stroke();
    context.fillStyle = SAMPLE.text;
    context.font = textFont(400);
    const baseline = field.y + field.height / 2 + TYPE_SIZE * 0.35;
    context.fillText(window.field, field.x + 10, baseline);
    context.fillStyle = SAMPLE.caret;
    context.fillRect(
      field.x + 11 + textWidth(window.field, textFont(400)),
      field.y + 6,
      1.5,
      field.height - 12,
    );
  }
  (window.paragraphs ?? []).forEach((paragraph, index) => {
    context.fillStyle = index === 0 ? SAMPLE.heading : SAMPLE.text;
    context.font = textFont(index === 0 ? 600 : 400);
    context.fillText(paragraph, ...parts.paragraphs[index]);
  });
  parts.lines.forEach((line, index) => {
    context.fillStyle = index === 0 ? SAMPLE.lineStrong : SAMPLE.line;
    context.beginPath();
    context.roundRect(line.x, line.y, line.width, line.height, 3);
    context.fill();
  });
  if (parts.chart) paintChart(context, parts.chart);
  if (parts.button && window.button) {
    const { button } = parts;
    context.fillStyle = SAMPLE.accent;
    context.beginPath();
    context.roundRect(button.x, button.y, button.width, button.height, 6);
    context.fill();
    context.fillStyle = SAMPLE.onAccent;
    context.font = buttonFont;
    context.textAlign = "center";
    context.fillText(
      window.button,
      button.x + button.width / 2,
      button.y + button.height / 2 + (TYPE_SIZE - 1) * 0.35,
    );
    context.textAlign = "start";
  }
  context.restore();
  return parts;
}
