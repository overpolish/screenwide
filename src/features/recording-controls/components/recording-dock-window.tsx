// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useRef } from "react";

import { useGeneralSettings } from "../../settings/use-general-settings";
import {
  cancelRecording,
  finishRecordingDockDrag,
  pauseRecording,
  recordingDockPainted,
  resizeRecordingDock,
  resumeRecording,
  stopRecording,
} from "../api";
import { selectSnapshot, useRecordingStore } from "../store";
import { useElapsedTime } from "../use-elapsed-time";
import { useRecordingMonitor } from "../use-recording-monitor";

import { RecordingDock } from "./recording-dock";

const report = (action: string) => (error: unknown) => {
  console.error(`Could not ${action} the recording`, error);
};

export function RecordingDockWindow() {
  const settings = useGeneralSettings();
  // The checks are on until the stored settings say otherwise.
  const showConfidenceChecks = settings?.showRecordingConfidenceChecks ?? true;
  const snapshot = useRecordingStore(selectSnapshot);
  const elapsedMs = useElapsedTime(snapshot);
  const monitor = useRecordingMonitor(showConfidenceChecks);

  // The pill's first show stays transparent until this page has drawn. The
  // second animation frame after becoming visible runs once the first has
  // reached the screen, so that is when the report goes out.
  useEffect(() => {
    let frame = 0;
    const reportPainted = () => {
      if (document.visibilityState !== "visible") return;
      window.cancelAnimationFrame(frame);
      frame = window.requestAnimationFrame(() => {
        frame = window.requestAnimationFrame(() => {
          recordingDockPainted().catch((error: unknown) => {
            console.error("Could not reveal the recording pill", error);
          });
        });
      });
    };
    reportPainted();
    document.addEventListener("visibilitychange", reportPainted);
    return () => {
      window.cancelAnimationFrame(frame);
      document.removeEventListener("visibilitychange", reportPainted);
    };
  }, []);

  const lastWidthRef = useRef(0);
  const resizeToContent = useCallback((width: number) => {
    if (width === lastWidthRef.current) return;
    lastWidthRef.current = width;
    resizeRecordingDock(width).catch(report("resize"));
  }, []);

  return (
    <RecordingDock
      countdownSeconds={snapshot.countdownSecondsRemaining}
      elapsedMs={elapsedMs}
      monitor={showConfidenceChecks ? monitor : undefined}
      onDiscard={() => {
        cancelRecording().catch(report("discard"));
      }}
      onPauseChange={(isPaused) => {
        const action = isPaused ? pauseRecording : resumeRecording;
        action().catch(report(isPaused ? "pause" : "resume"));
      }}
      onPointerUp={() => {
        void finishRecordingDockDrag();
      }}
      onStop={() => {
        stopRecording().catch(report("stop"));
      }}
      onWidthChange={resizeToContent}
      status={snapshot.status}
    />
  );
}
