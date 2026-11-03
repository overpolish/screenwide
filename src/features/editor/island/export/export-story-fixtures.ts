// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type {
  ToolPanelExport,
  ToolPanelExportCompare,
  ToolPanelExportQuality,
} from "../../tools/tool-panel-export";

// A two-minute screen recording at 2880 × 1800, measured at each look, as
// the editor publishes it to the Export island.

const QUALITY: ToolPanelExportQuality = {
  look: "balanced",
  looks: [
    { bytes: 9_400_000, height: 900, id: "smallest", width: 1440 },
    { bytes: 18_700_000, height: 1350, id: "smaller", width: 2160 },
    { bytes: 41_200_000, height: 1800, id: "balanced", width: 2880 },
    { bytes: 186_000_000, height: 1800, id: "original", width: 2880 },
  ],
};

/** The looks while they are still being measured. */
export const MEASURING_QUALITY: ToolPanelExportQuality = {
  ...QUALITY,
  looks: QUALITY.looks.map((look) => ({ ...look, bytes: null })),
};

/** The looks part-way through measuring: the smaller two have landed,
 * the rest are still being measured. */
export const PARTLY_MEASURED_QUALITY: ToolPanelExportQuality = {
  ...QUALITY,
  looks: QUALITY.looks.map((look, index) =>
    index < 2 ? look : { ...look, bytes: null },
  ),
};

/** A frame of a window with text, drawn at the size the backend writes. */
const frame = (blur: number) => ({
  canvas: { height: 900, width: 1440, x: 0, y: 0 },
  height: 900,
  src: `data:image/svg+xml,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="1440" height="900"><filter id="b"><feGaussianBlur stdDeviation="${blur.toString()}"/></filter><g filter="url(#b)"><rect width="1440" height="900" fill="#e8e4dc"/><rect x="80" y="60" width="1280" height="780" rx="16" fill="#fff"/>${Array.from(
      { length: 18 },
      (_, row) =>
        `<rect x="140" y="${(130 + row * 38).toString()}" width="${(600 + ((row * 97) % 520)).toString()}" height="14" rx="7" fill="#57534e"/>`,
    ).join("")}</g></svg>`,
  )}`,
  width: 1440,
});

const COMPARE: ToolPanelExportCompare = {
  exported: frame(1.2),
  failed: false,
  isUpdating: false,
  original: frame(0),
};

export const COMPARE_LOADING: ToolPanelExportCompare = {
  exported: null,
  failed: false,
  isUpdating: true,
  original: null,
};

export const VIDEO: ToolPanelExport = {
  cameraKey: null,
  cameraOutput: "combined",
  canExport: true,
  collapseAudio: null,
  compare: COMPARE,
  deleteProjectAfterExport: false,
  directory: "/Users/sam/Movies/Screenwide",
  exported: null,
  extension: "mp4",
  failure: null,
  fileStem: "Onboarding walkthrough",
  kind: "video",
  progress: null,
  quality: QUALITY,
  revealWhenDone: true,
  screenshotFormat: null,
  subtitles: "srt",
};

export const SCREENSHOT: ToolPanelExport = {
  ...VIDEO,
  cameraOutput: null,
  compare: null,
  directory: "/Users/sam/Pictures/Screenwide",
  extension: "png",
  fileStem: "Pricing page",
  kind: "screenshot",
  quality: null,
  screenshotFormat: "png",
  subtitles: null,
};

export const AUDIO: ToolPanelExport = {
  ...VIDEO,
  cameraOutput: null,
  collapseAudio: false,
  compare: null,
  extension: "m4a",
  fileStem: "Standup notes",
  kind: "audio",
  quality: null,
};
