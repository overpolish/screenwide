// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { CameraBackgroundRange } from "../../../../bindings/CameraBackgroundRange";

export type { CameraBackgroundRange };

/** Shorter than this, a range would be over before its edges faded in. */
export const CAMERA_BACKGROUND_RANGE_MINIMUM_MS = 1_000;
