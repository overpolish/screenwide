// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";

import { selectStatus, useRecordingStore } from "../recording-controls/store";
import {
  hideRegionSelector,
  listMonitors,
  prepareScreenshotRegionMagnifier,
  setRecordingControlsOpacity,
  setRegionSelectorPassthrough,
  showRegionSelector,
} from "../recording-sources/api";
import { useRecordingSourceStore } from "../recording-sources/store";
import { MonitorDetails } from "../recording-sources/types";

import { EMPTY_REGION, fitRegion, snapRegion } from "./region-geometry";
import {
  captureScreenshotRegion,
  endScreenshotCapture,
  screenshotCaptureDestination,
} from "./screenshot-session";
import { useNativeScreenshotRegion } from "./use-native-screenshot-region";
import { useRegionGestureVisibility } from "./use-region-gesture-visibility";
import { useScreenshotShortcut } from "./use-screenshot-shortcut";

const EMPTY_BOUNDS = { height: 0, width: 0 };
const SCREENSHOT_DISMISS_REQUESTED_EVENT =
  "screenshot-region://dismiss-requested";

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

  const activeMonitor = isScreenshotSurface
    ? screenshotMonitor
    : recordingMode === "region"
      ? selectedMonitor
      : null;
  useEffect(() => {
    nativeMonitorRef.current = activeMonitor?.id ?? null;
  }, [activeMonitor?.id, isScreenshotSurface]);

  const { beginGesture, finishGesture } =
    useRegionGestureVisibility(gestureActive);
  const nativeOscEnabled =
    !!activeMonitor && (isScreenshotSurface || recordingMode === "region");

  const nativeOscAvailable = useNativeScreenshotRegion({
    allowDrawing: isScreenshotSurface,
    aspect: isScreenshotSurface ? undefined : regionAspectRatio,
    bounds: activeMonitor?.size ?? EMPTY_BOUNDS,
    desktop: true,
    enabled: nativeOscEnabled,
    inputEnabled: isIdle,
    monitorId: activeMonitor?.id,
    onFinished: (nextRegion, gesture, monitorId) => {
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
      setDraft(snapped);
      setRegion(snapped);
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
    onMonitorChange: (monitorId) => {
      if (nativeMonitorRef.current === monitorId) return;
      nativeMonitorRef.current = monitorId;
      void listMonitors()
        .then((monitors) => {
          const monitor = monitors.find(
            (candidate) => candidate.id === monitorId,
          );
          if (!monitor) return;
          if (isScreenshotSurface) setScreenshotMonitor(monitor);
          else setSelectedMonitor(monitor);
        })
        .catch((error: unknown) => {
          nativeMonitorRef.current = activeMonitor?.id ?? null;
          console.error("Could not follow the Region monitor", error);
        });
    },
    onReconciled: (nextRegion) => {
      setDraft(nextRegion);
      if (!isScreenshotSurface) setRegion(nextRegion);
    },
    onRegionChange: setDraft,
    region: draft,
    showFrame: isScreenshotSurface || isIdle,
    showHandles: !isScreenshotSurface && isIdle,
    visible: isScreenshotSurface ? isIdle : nativeOscEnabled,
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
    let unlisten: UnlistenFn | undefined;
    let disposed = false;
    let cancelling = false;
    const cancel = () => {
      if (!useRecordingSourceStore.getState().isScreenshotCapture) return;
      if (cancelling) return;
      cancelling = true;
      // Escape closes only the borrowed screenshot overlay. The ruler was
      // already open before this session and remains available underneath it.
      void endScreenshotCapture()
        .catch((error: unknown) => {
          console.error("Could not cancel the screenshot session", error);
        })
        .finally(() => {
          cancelling = false;
        });
    };
    const cancelKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") cancel();
    };

    window.addEventListener("keydown", cancelKey);
    void listen(SCREENSHOT_DISMISS_REQUESTED_EVENT, cancel).then((listener) => {
      if (disposed) listener();
      else unlisten = listener;
    });

    return () => {
      disposed = true;
      window.removeEventListener("keydown", cancelKey);
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    void setRecordingControlsOpacity(isScreenshotCapture ? 0 : 1);
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

  useEffect(() => {
    if (!activeMonitor) return;

    void setRegionSelectorPassthrough(!isIdle);
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
  ]);

  if (!activeMonitor) return null;

  return (
    <main className="relative h-screen w-screen overflow-hidden select-none" />
  );
}
