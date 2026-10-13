// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import {
  RECORDING_CAPTIONS_EVENT,
  recordingCaptions,
} from "../../recording/captions/captions-api";

import type { Caption } from "../../../../bindings/Caption";

const NONE: Caption[] = [];

/** The captions last made of the recording `artifactId`, on or off, read
 * again each time the editor makes them; none until they are first made.
 * They are read rather than mirrored, so a long recording's never ride the
 * settings snapshot. */
export function useCaptionList(artifactId: number | undefined) {
  const [read, setRead] = useState<{
    artifactId: number;
    captions: Caption[];
  } | null>(null);
  useEffect(() => {
    if (artifactId === undefined) return;
    let disposed = false;
    let asked = 0;
    let stop: (() => void) | undefined;
    const ask = () => {
      const asking = ++asked;
      recordingCaptions(artifactId)
        .then((captions) => {
          if (!disposed && asking === asked) setRead({ artifactId, captions });
        })
        .catch((cause: unknown) => {
          console.error("Could not read the captions", cause);
        });
    };
    ask();
    listen<{ artifactId: number }>(RECORDING_CAPTIONS_EVENT, (event) => {
      if (event.payload.artifactId === artifactId) ask();
    })
      .then((unlisten) => {
        if (disposed) unlisten();
        else stop = unlisten;
      })
      .catch((cause: unknown) => {
        console.error("Could not follow the captions", cause);
      });
    return () => {
      disposed = true;
      stop?.();
    };
  }, [artifactId]);
  return read && read.artifactId === artifactId ? read.captions : NONE;
}
