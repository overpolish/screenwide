// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import { getRecordingPreview } from "../api";
import { RecordingPreview } from "../types";

export function useRecordingEditorPreview({
  artifactId,
  shouldPrepare,
}: {
  artifactId: number | undefined;
  shouldPrepare: boolean;
}) {
  const [state, setState] = useState<{
    artifactId: number;
    error: string | null;
    preview: RecordingPreview | null;
  } | null>(null);

  useEffect(() => {
    if (!shouldPrepare || artifactId === undefined) return;

    let disposed = false;
    void getRecordingPreview(artifactId)
      .then((preview) => {
        if (!disposed) {
          setState({ artifactId, error: null, preview });
        }
      })
      .catch((cause: unknown) => {
        if (disposed) return;
        console.error("Could not prepare the recording preview", cause);
        setState({
          artifactId,
          error: cause instanceof Error ? cause.message : String(cause),
          preview: null,
        });
      });

    // Reduce noise changes what a track sounds like, and with it the
    // waveform: the app hands the waveforms over again when it does.
    let stop: (() => void) | undefined;
    listen<RecordingPreview>("editor://recording-preview", (event) => {
      if (event.payload.artifactId !== artifactId) return;
      setState({ artifactId, error: null, preview: event.payload });
    })
      .then((unlisten) => {
        if (disposed) unlisten();
        else stop = unlisten;
      })
      .catch((cause: unknown) => {
        console.error("Could not follow the recording's waveforms", cause);
      });

    return () => {
      disposed = true;
      stop?.();
    };
  }, [artifactId, shouldPrepare]);

  const current =
    shouldPrepare && state?.artifactId === artifactId ? state : null;
  return {
    error: current?.error ?? null,
    isPreparing: shouldPrepare && current === null,
    preview: current?.preview ?? null,
  };
}
