// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

export const FILE_NAME = "Screenwide 2026-08-08 at 14.32.05";

export type QualityStep = { dimensions: string; label: string; size: string };

// One ladder in place of Output size and Compression: each step is a size
// and a compression the app already offers, named by what it gives.
export const SCREEN_STEPS: QualityStep[] = [
  { dimensions: "1280 × 720", label: "Smallest", size: "12 MB" },
  { dimensions: "1920 × 1080", label: "Smaller", size: "31 MB" },
  { dimensions: "3840 × 2160", label: "Balanced", size: "74 MB" },
  { dimensions: "3840 × 2160", label: "Original", size: "182 MB" },
];

export const CAMERA_STEPS: QualityStep[] = [
  { dimensions: "640 × 360", label: "Smallest", size: "4 MB" },
  { dimensions: "1280 × 720", label: "Smaller", size: "9 MB" },
  { dimensions: "1920 × 1080", label: "Balanced", size: "22 MB" },
  { dimensions: "1920 × 1080", label: "Original", size: "58 MB" },
];

export const BALANCED = 2;
