// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import {
  SCROLLING_CAPTURE_FINISHED_EVENT,
  SCROLLING_CAPTURE_PROGRESS_EVENT,
  ScrollingCaptureProgressEvent,
} from "./scrolling-capture-events";
import { ScrollingCaptureOverlay } from "./scrolling-capture-overlay";

export function ScrollingCaptureOverlayWindow() {
  const [progress, setProgress] = useState<ScrollingCaptureProgressEvent>();
  const [finished, setFinished] = useState(false);
  // Read from the URL rather than an event: the capture starts emitting before
  // this window has loaded, so the earliest events never reach the listener.
  const cancellable =
    new URLSearchParams(window.location.search).get("cancellable") === "1";

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void)[] = [];

    void Promise.all([
      listen<ScrollingCaptureProgressEvent>(
        SCROLLING_CAPTURE_PROGRESS_EVENT,
        ({ payload }) => {
          setProgress(payload);
        },
      ),
      listen(SCROLLING_CAPTURE_FINISHED_EVENT, () => {
        setFinished(true);
      }),
    ]).then((stopListening) => {
      if (disposed) for (const stop of stopListening) stop();
      else unlisten = stopListening;
    });

    return () => {
      disposed = true;
      for (const stop of unlisten) stop();
    };
  }, []);

  // The backend closes this window once the capture settles; until it does, the
  // card holds a terminal label rather than spinning on a phase that is over.
  return (
    <ScrollingCaptureOverlay
      cancellable={cancellable}
      finished={finished}
      phase={progress?.phase}
    />
  );
}
