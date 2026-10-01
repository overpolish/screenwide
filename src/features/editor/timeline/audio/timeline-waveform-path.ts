// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { decibelGain } from "./audio-level";

export const timelineWaveformPath = (
  points: number[],
  volumeDecibels: number,
) => {
  if (points.length === 0) return "";
  const center = 20;
  return points
    .map((peak, index) => {
      const x = (index / Math.max(1, points.length - 1)) * 1000;
      const adjustedPeak = Math.min(1, peak * decibelGain(volumeDecibels));
      const height = Math.max(
        1.25,
        Math.pow(Math.max(0, adjustedPeak), 0.55) * 18.5,
      );
      return `M${x.toFixed(2)} ${(center - height).toFixed(2)}V${(center + height).toFixed(2)}`;
    })
    .join(" ");
};
