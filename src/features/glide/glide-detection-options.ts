// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { defaultGlideFoldOptions, type GlideFoldOptions } from "./glide-folds";

import type { GlideRestOptions } from "./glide-settling";

export type GlideDetectorOptions = GlideFoldOptions &
  GlideRestOptions & {
    openingGraceMs: number;
    reversalHysteresis: number;
  };

export const defaultGlideDetectorOptions: GlideDetectorOptions = {
  ...defaultGlideFoldOptions,
  motionNoiseFloor: 2,
  openingGraceMs: 30,
  restMs: 100,
  reversalHysteresis: 10,
};
