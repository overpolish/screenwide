// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The export's quality as four looks, named by what they give rather than by
// setting, smallest first: each is a size and a compression the export
// already takes. Original keeps both as recorded; Balanced compresses the
// full size; Smaller and Smallest also shrink it, to three quarters and to
// half. A camera saved as a file of its own takes the same look on its own
// scales. Settings that match no look were customised.

export type ExportLook = "balanced" | "original" | "smaller" | "smallest";

export const EXPORT_LOOKS: readonly ExportLook[] = [
  "smallest",
  "smaller",
  "balanced",
  "original",
];

/** One video's quality: its scale in percent, and its compression, 0
 * (Original) to 4 (Smallest file). */
export type VideoQuality = {
  compression: number;
  resolutionScalePercent: number;
};

/** The quality `look` gives a video whose scales, the original first, are
 * `scales`. A video with fewer scales keeps the smallest it has. */
export const lookQuality = (
  look: ExportLook,
  scales: readonly number[],
): VideoQuality => {
  const original = scales[0] ?? 100;
  const threeQuarters = scales[1] ?? original;
  const half = scales[2] ?? threeQuarters;
  switch (look) {
    case "original":
      return { compression: 0, resolutionScalePercent: original };
    case "balanced":
      return { compression: 2, resolutionScalePercent: original };
    case "smaller":
      return { compression: 3, resolutionScalePercent: threeQuarters };
    case "smallest":
      return { compression: 4, resolutionScalePercent: half };
  }
};

const isQuality = (a: VideoQuality, b: VideoQuality) =>
  a.compression === b.compression &&
  a.resolutionScalePercent === b.resolutionScalePercent;

/** The look the screen's quality, and a separate camera's where there is
 * one, both match, or null where they were customised. */
export const qualityLook = (
  screen: { quality: VideoQuality; scales: readonly number[] },
  camera: { quality: VideoQuality; scales: readonly number[] } | null,
): ExportLook | null =>
  EXPORT_LOOKS.find(
    (look) =>
      isQuality(lookQuality(look, screen.scales), screen.quality) &&
      (camera === null ||
        isQuality(lookQuality(look, camera.scales), camera.quality)),
  ) ?? null;
