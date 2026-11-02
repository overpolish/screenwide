// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";

import { paintIsland } from "./sample-island";
import { paintSystemBars } from "./sample-system-bars";
import { paintAppWindow } from "./sample-window";
import { type AppWindow, windowParts } from "./sample-window-layout";
import {
  paintWallpaper,
  type Painter,
  type RecordingScene,
  type Spot,
} from "./scene-kit";

import type { PointerKey } from "../../../../bindings/PointerKey";
import type { PointerScript } from "../../../../bindings/PointerScript";
import type { PictureSize } from "../settings-preview-api";

// The recordings Settings shows a setting's effect on, composed by the
// app's compositor from a desktop the page paints and a scripted pointer:
// a display's worth of points at a display's shape, so the cursor is drawn
// at its real size. Someone types a line into a note, then publishes from
// a dashboard: auto zoom closes in on the typing, then on the button.

const DESKTOP: PictureSize = { height: 450, width: 800 };

const note = (): AppWindow => ({
  box: { height: 290, width: 380, x: 64, y: 62 },
  field: t("settings-preview-note-first"),
  heading: t("settings-preview-note-title"),
  lines: [0.92, 0.78, 0.86, 0.64, 0.8, 0.5],
  sidebar: true,
});

const dashboard = (): AppWindow => ({
  box: { height: 240, width: 330, x: 404, y: 132 },
  button: t("settings-preview-window-button"),
  chart: true,
  heading: t("settings-preview-dashboard-title"),
});

const paintDesktop: Painter = (context, size, { notch, wallpaper }) => {
  paintWallpaper(context, size, wallpaper);
  paintSystemBars(context, size, notch);
  paintAppWindow(context, note());
  paintAppWindow(context, dashboard());
};

const key = (atMs: number, [x, y]: Spot, pressed = false) =>
  ({ atMs, pressed, x, y }) satisfies PointerKey;

/** A click at `atMs`: pressed, then let go a beat later. */
const click = (atMs: number, at: Spot) => [
  key(atMs, at, true),
  key(atMs + 90, at),
];

/** Into the note's field, a line typed, then across to Publish. */
function workScript(): PointerScript {
  const field = windowParts(note()).field;
  const button = windowParts(dashboard()).button;
  if (!field || !button) throw new Error("The sample windows lost a part");
  const typing: Spot = [
    field.x + field.width * 0.7,
    field.y + field.height / 2,
  ];
  const publish: Spot = [
    button.x + button.width / 2,
    button.y + button.height / 2,
  ];
  const typed = Array.from({ length: 16 }, (_, index) => 1_400 + index * 140);
  return {
    durationMs: 9_000,
    keys: [
      key(0, [420, 420]),
      key(1_000, typing),
      ...click(1_100, typing),
      key(4_600, typing),
      key(5_500, publish),
      ...click(5_600, publish),
      key(6_200, [publish[0] - 30, publish[1] + 36]),
      key(9_000, [publish[0] - 30, publish[1] + 36]),
    ],
    typingMs: typed,
  };
}

export const AUTO_ZOOM_SCENE: RecordingScene = {
  kind: "recording",
  paint: paintDesktop,
  get script() {
    return workScript();
  },
  size: DESKTOP,
};

/** A quick sweep back and forth, where a lower frame rate shows. */
export const FRAME_RATE_SCENE: RecordingScene = {
  kind: "recording",
  paint: paintDesktop,
  script: {
    durationMs: 3_000,
    keys: [key(0, [140, 380]), key(1_500, [660, 150]), key(3_000, [140, 380])],
    typingMs: [],
  },
  size: DESKTOP,
};

/** The desktop recorded with Screenwide's island on it, or without. */
export const INCLUDE_SCREENWIDE_SCENE: RecordingScene = {
  kind: "recording",
  paint: paintDesktop,
  paintScreenwide: paintIsland,
  size: DESKTOP,
};
