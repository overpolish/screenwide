// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { getRecordingMoments, RecordingMoment } from "./recording-moments";

const NO_MOMENTS: RecordingMoment[] = [];

/** The moments placed in the recording open as `artifactId`. They never
 * change once recorded, so they are read once per recording. */
export function useRecordingMoments(artifactId: number) {
  const [moments, setMoments] = useState<{
    artifactId: number;
    moments: RecordingMoment[];
  } | null>(null);

  useEffect(() => {
    let current = true;
    getRecordingMoments(artifactId)
      .then((loaded) => {
        if (current) setMoments({ artifactId, moments: loaded });
      })
      .catch((error: unknown) => {
        console.error("Could not read the recording's moments", error);
      });
    return () => {
      current = false;
    };
  }, [artifactId]);

  return moments?.artifactId === artifactId ? moments.moments : NO_MOMENTS;
}
