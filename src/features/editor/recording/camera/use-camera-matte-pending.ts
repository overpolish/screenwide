// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

import { getRecordingCameraEffects } from "./camera-effects-api";
import { useCameraEffectsRevision } from "./use-camera-effects-revision";

import type { PendingStretch } from "../../../../bindings/PendingStretch";

const NOTHING_PENDING: readonly PendingStretch[] = [];

/**
 * The stretches of the edited video the camera's background ranges cover
 * whose matte is still being made, read again as the ranges or the timeline
 * change and as each chunk of the matte is made.
 */
export function useCameraMattePending(
  artifactId: number,
  edit: RecordingTimelineEdit,
): readonly PendingStretch[] {
  const [pending, setPending] = useState<{
    artifactId: number;
    stretches: readonly PendingStretch[];
  } | null>(null);
  const revision = useCameraEffectsRevision(artifactId);
  const { cameraBackgroundRanges, segments } = edit;
  useEffect(() => {
    if (!cameraBackgroundRanges) return;
    let current = true;
    getRecordingCameraEffects(artifactId)
      .then((effects) => {
        if (current) setPending({ artifactId, stretches: effects.pending });
      })
      .catch(() => {
        // A recording without a camera has nothing to make.
      });
    return () => {
      current = false;
    };
  }, [artifactId, cameraBackgroundRanges, segments, revision]);
  return cameraBackgroundRanges && pending?.artifactId === artifactId
    ? pending.stretches
    : NOTHING_PENDING;
}
