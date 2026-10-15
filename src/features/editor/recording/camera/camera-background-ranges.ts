// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";

import type { CameraBackgroundRange } from "../../../../bindings/CameraBackgroundRange";

export type { CameraBackgroundRange };

/** Shorter than this, a range would be over before its edges faded in. */
export const CAMERA_BACKGROUND_RANGE_MINIMUM_MS = 1_000;

/** What a range is called on the timeline. */
export const cameraBackgroundRangeLabel = ({
  background,
}: CameraBackgroundRange) =>
  t("editor-timeline-camera-background-range", {
    background:
      background === "blurred"
        ? "blur"
        : background === "image"
          ? "picture"
          : "none",
  });
