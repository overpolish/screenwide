// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import {
  getRecordingMoments,
  listenToNoteTranscripts,
  RecordingMoment,
} from "./recording-moments";

const NO_MOMENTS: RecordingMoment[] = [];

/** The moments placed in the recording open as `artifactId`. The moments
 * themselves never change once recorded; they are read again only as their
 * notes' transcripts move on. */
export function useRecordingMoments(artifactId: number) {
  const [moments, setMoments] = useState<{
    artifactId: number;
    moments: RecordingMoment[];
  } | null>(null);

  useEffect(() => {
    let current = true;
    let stop: (() => void) | undefined;
    const load = () => {
      getRecordingMoments(artifactId)
        .then((loaded) => {
          if (current) setMoments({ artifactId, moments: loaded });
        })
        .catch((error: unknown) => {
          console.error("Could not read the recording's moments", error);
        });
    };
    load();
    void listenToNoteTranscripts(load)
      .then((unlisten) => {
        if (current) stop = unlisten;
        else unlisten();
      })
      .catch(() => undefined);
    return () => {
      current = false;
      stop?.();
    };
  }, [artifactId]);

  return moments?.artifactId === artifactId ? moments.moments : NO_MOMENTS;
}
