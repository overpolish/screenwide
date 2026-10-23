// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useRef, useState } from "react";

import { selectStatus, useRecordingStore } from "../recording-controls/store";
import {
  hideRegionSelector,
  listMonitors,
  prepareScreenshotRegionMagnifier,
  setRecordingControlsBorrowed,
  setRegionSelectorPassthrough,
  showRegionSelector,
  showWindowPickBackdrop,
} from "../recording-sources/api";
import { useRecordingSourceStore } from "../recording-sources/store";
import { MonitorDetails } from "../recording-sources/types";

import { EMPTY_REGION, fitRegion, snapRegion } from "./region-geometry";
import {
  captureScreenshotRegion,
  screenshotCaptureDestination,
} from "./screenshot-session";
import { useNativeScreenshotRegion } from "./use-native-screenshot-region";
import { useRegionGestureVisibility } from "./use-region-gesture-visibility";
import { useScreenshotCancel } from "./use-screenshot-cancel";
import { useScreenshotShortcut } from "./use-screenshot-shortcut";
import { useWindowPickOverlay } from "./use-window-pick-overlay";

const EMPTY_BOUNDS = { height: 0, width: 0 };

export function RegionSelectorWindow() {
  const {
    isScreenshotCapture,
    recordingMode,
    region,
    regionAspectRatio,
    selectedMonitor,
    setRegion,
    setSelectedMonitor,
  } = useRecordingSourceStore((state) => state);
  const recordingStatus = useRecordingStore(selectStatus);
  const [screenshotMonitor, setScreenshotMonitor] =
    useState<MonitorDetails | null>(null);
  const isScreenshotSurface = isScreenshotCapture;
  const [draft, setDraft] = useState(
    isScreenshotSurface ? EMPTY_REGION : region,
  );
  const [seededForSession, setSeededForSession] = useState(isScreenshotSurface);
  if (seededForSession !== isScreenshotSurface) {
    // Reseed while rendering rather than in an effect: otherwise the session
    // would leave one native sync frame of the recording region before the
    // empty draft landed - a visible flash during screenshot activation.
    setSeededForSession(isScreenshotSurface);
    setDraft(isScreenshotSurface ? EMPTY_REGION : region);
  }
  const [gestureActive, setGestureActive] = useState(false);
  const [captureRequest, setCaptureRequest] = useState(0);
  const resizeActiveRef = useRef(false);
  // A capture ends the session, so whichever gesture starts one shuts the
  // others out until the session is over.
  const isCapturingRef = useRef(false);
  const nativeMonitorRef = useRef<number | null>(null);
  const isIdle = recordingStatus === "idle";
  useScreenshotShortcut();
  useScreenshotCancel();
  // Picking a window borrows the overlay to outline the one under the
  // pointer, with nothing to edit.
  const windowPick = useWindowPickOverlay();
  const { isPicking } = windowPick;

  const commitRecordingRegion = useCallback(
    (nextRegion: typeof region, monitorId?: number) => {
      const commit = () => {
        setDraft(nextRegion);
        setRegion(nextRegion);
      };
      if (monitorId === undefined || nativeMonitorRef.current === monitorId) {
        commit();
        return;
      }
      void listMonitors()
        .then((monitors) => {
          const monitor = monitors.find(
            (candidate) => candidate.id === monitorId,
          );
          if (!monitor) return;
          // Commit the new monitor and its monitor-local rectangle in one turn;
          // publishing either half first can briefly re-clamp the marquee to
          // the previous anchor.
          nativeMonitorRef.current = monitorId;
          setSelectedMonitor(monitor);
          commit();
        })
        .catch((error: unknown) => {
          console.error("Could not follow the Region monitor", error);
        });
    },
    [setRegion, setSelectedMonitor],
  );

  const activeMonitor = isScreenshotSurface
    ? screenshotMonitor
    : isPicking
      ? (windowPick.monitor ?? selectedMonitor)
      : recordingMode === "region"
        ? selectedMonitor
        : null;
  useEffect(() => {
    nativeMonitorRef.current = activeMonitor?.id ?? null;
  }, [activeMonitor?.id, isScreenshotSurface]);

  const { beginGesture, finishGesture } =
    useRegionGestureVisibility(gestureActive);
  const nativeOscEnabled =
    !!activeMonitor &&
    (isScreenshotSurface || isPicking || recordingMode === "region");
  const picked = windowPick.outline;

  const nativeOscAvailable = useNativeScreenshotRegion({
    allowDrawing: isScreenshotSurface,
    aspect: isScreenshotSurface || isPicking ? undefined : regionAspectRatio,
    bounds: activeMonitor?.size ?? EMPTY_BOUNDS,
    desktop: true,
    enabled: nativeOscEnabled,
    inputEnabled: isIdle && !isPicking,
    monitorId: activeMonitor?.id,
    onFinished: (nextRegion, gesture, monitorId) => {
      if (isPicking) return;
      const snapped = snapRegion(nextRegion);
      if (isScreenshotSurface) {
        if (gesture !== "drawing" || !activeMonitor || isCapturingRef.current)
          return;
        isCapturingRef.current = true;
        captureScreenshotRegion(
          screenshotCaptureDestination(),
          monitorId ?? activeMonitor.id,
          snapped,
        );
        return;
      }
      commitRecordingRegion(snapped, monitorId);
    },
    onGesture: ({ dragging, drawing, resizeDirection: direction }) => {
      const resizing = direction !== undefined;
      if (resizing && !resizeActiveRef.current)
        setCaptureRequest((current) => current + 1);
      resizeActiveRef.current = resizing;
      const active = dragging || drawing || resizing;
      setGestureActive(active);
      if (!isScreenshotSurface) {
        if (active) beginGesture();
        else finishGesture();
      }
    },
    onReconciled: (nextRegion, monitorId) => {
      if (isPicking) return;
      if (isScreenshotSurface) {
        setDraft(nextRegion);
      } else {
        commitRecordingRegion(nextRegion, monitorId);
      }
    },
    onRegionChange: setDraft,
    region: picked
      ? {
          position: { x: picked.x, y: picked.y },
          size: { height: picked.height, width: picked.width },
        }
      : draft,
    showFrame:
      isPicking ||
      isScreenshotSurface ||
      (recordingMode === "region" && isIdle),
    showHandles: !isPicking && !isScreenshotSurface && isIdle,
    visible: isPicking
      ? windowPick.hovered !== null
      : isScreenshotSurface
        ? isIdle
        : nativeOscEnabled,
    windowHighlight: isPicking,
    windowLabel: nativeOscEnabled ? getCurrentWindow().label : undefined,
  });

  useEffect(() => {
    if (!isScreenshotSurface) {
      // eslint-disable-next-line @eslint-react/set-state-in-effect
      setScreenshotMonitor(null);
      return;
    }

    let disposed = false;
    void listMonitors()
      .then((monitors) => {
        if (disposed) return;
        setScreenshotMonitor(
          monitors.find((monitor) => monitor.id === selectedMonitor?.id) ??
            monitors.find((monitor) => monitor.isPrimary) ??
            monitors.find((_monitor, index) => index === 0) ??
            null,
        );
      })
      .catch((error: unknown) => {
        console.error("Could not resolve the screenshot monitor", error);
      });
    return () => {
      disposed = true;
    };
  }, [isScreenshotSurface, selectedMonitor?.id]);

  useEffect(() => {
    // Cross-window storage updates replace the persisted region. A screenshot
    // session starts from nothing instead: the region is drawn for that shot
    // alone, and the persisted one comes back when the session ends.
    // eslint-disable-next-line @eslint-react/set-state-in-effect
    setDraft(isScreenshotSurface ? EMPTY_REGION : region);
  }, [isScreenshotSurface, region]);

  useEffect(() => {
    // Each session gets its one capture back.
    isCapturingRef.current = false;
  }, [isScreenshotSurface]);

  useEffect(() => {
    void setRecordingControlsBorrowed(isScreenshotCapture);
  }, [isScreenshotCapture]);

  useEffect(() => {
    if (!activeMonitor) {
      if (isScreenshotSurface) return;
      void setRegionSelectorPassthrough(true);
      void hideRegionSelector();
      return;
    }

    const fitted = isScreenshotSurface
      ? fitRegion(region, activeMonitor.size.width, activeMonitor.size.height)
      : region;
    // The overlay keeps a local draft so dragging does not write storage per
    // frame. A screenshot session has no region until one is drawn, so its
    // draft starts empty rather than from the recording region.
    // eslint-disable-next-line @eslint-react/set-state-in-effect
    setDraft(isScreenshotSurface ? EMPTY_REGION : fitted);
    if (
      !isScreenshotSurface &&
      JSON.stringify(fitted) !== JSON.stringify(region)
    ) {
      setRegion(fitted);
    }
    void showRegionSelector(activeMonitor, true);
  }, [activeMonitor, isScreenshotSurface, region, setRegion]);

  // The frozen desktop goes behind a pick once the native overlay is up.
  useEffect(() => {
    if (!isPicking || !nativeOscAvailable) return;
    void showWindowPickBackdrop();
  }, [isPicking, nativeOscAvailable]);

  useEffect(() => {
    if (!activeMonitor) return;

    // Fading out after a pick, the overlay already lets clicks through to the
    // window picked.
    void setRegionSelectorPassthrough(!isIdle || windowPick.isLeaving);
    if (!isIdle || captureRequest === 0 || !nativeOscAvailable) return;
    void prepareScreenshotRegionMagnifier(
      activeMonitor.id,
      getCurrentWindow().label,
    ).catch((reason: unknown) => {
      console.error("Could not prepare the native region magnifier", reason);
    });
  }, [
    activeMonitor,
    captureRequest,
    isIdle,
    isScreenshotSurface,
    nativeOscAvailable,
    windowPick.isLeaving,
  ]);

  if (!activeMonitor) return null;

  return (
    <main className="relative h-screen w-screen overflow-hidden select-none" />
  );
}
