// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Channel } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";

import { cn } from "../../lib/styling";
import { selectStatus, useRecordingStore } from "../recording-controls/store";
import {
  hideRegionSelector,
  listMonitors,
  setRecordingControlsOpacity,
  setRegionSelectorPassthrough,
  showRegionSelector,
  takeMonitorScreenshot,
} from "../recording-sources/api";
import { useRecordingSourceStore } from "../recording-sources/store";
import { MonitorDetails, Region } from "../recording-sources/types";
import { ScreenshotDestination } from "../screenshots/api";
import { useGeneralSettings } from "../settings/use-general-settings";

import { Magnifier } from "./magnifier";
import { RegionDrawingSurface } from "./region-drawing-surface";
import {
  EMPTY_REGION,
  fitRegion,
  hasRegion,
  snapRegion,
  wholePixel,
} from "./region-geometry";
import { RegionShade } from "./region-shade";
import { RegionTransformFrame } from "./region-transform-frame";
import { ScreenshotRegionControls } from "./screenshot-region-controls";
import {
  captureScreenshotRegion,
  endScreenshotCapture,
} from "./screenshot-session";
import { ResizeDirection } from "./types";
import { useKeyHeld } from "./use-key-held";
import { useRegionGestureVisibility } from "./use-region-gesture-visibility";
import { useScreenshotShortcut } from "./use-screenshot-shortcut";

// Holding this ignores the linked ratio, so the region can be reshaped
// freely; the shape it ends up with becomes the new ratio.
const FREE_ASPECT_KEY = "Shift";

const screenshotParameters = new URLSearchParams(window.location.search);
const requestedMonitorId = screenshotParameters.get("monitorId");
const parsedMonitorId =
  requestedMonitorId === null ? NaN : Number(requestedMonitorId);
const screenshotMonitorId = Number.isFinite(parsedMonitorId)
  ? parsedMonitorId
  : null;
const screenshotDestination: ScreenshotDestination =
  screenshotParameters.get("destination") === "clipboard"
    ? "clipboard"
    : "export";
const isScreenshotWindow = screenshotMonitorId !== null;

export function RegionSelectorWindow() {
  const {
    isScreenshotCapture,
    recordingMode,
    region,
    regionAspectRatio,
    selectedMonitor,
    setRegion,
  } = useRecordingSourceStore((state) => state);
  const recordingStatus = useRecordingStore(selectStatus);
  const [screenshotMonitor, setScreenshotMonitor] =
    useState<MonitorDetails | null>(null);
  const isScreenshotSurface = isScreenshotCapture && isScreenshotWindow;
  const [draft, setDraft] = useState(region);
  const [seededForSession, setSeededForSession] = useState(isScreenshotSurface);
  if (seededForSession !== isScreenshotSurface) {
    // Reseed while rendering rather than in an effect: otherwise the session
    // would leave one painted frame of the recording marquee before the empty
    // draft landed - a visible flash when the overlay was already on screen.
    setSeededForSession(isScreenshotSurface);
    setDraft(isScreenshotSurface ? EMPTY_REGION : region);
  }
  const [screenshotAspect, setScreenshotAspect] = useState<number>();
  const [resizeDirection, setResizeDirection] = useState<ResizeDirection>();
  const [isDragging, setIsDragging] = useState(false);
  const [isDrawing, setIsDrawing] = useState(false);
  const [screenshot, setScreenshot] = useState<{
    height: number;
    pixels: ArrayBuffer;
    width: number;
  } | null>(null);
  const [captureRequest, setCaptureRequest] = useState(0);
  const freeAspect = useKeyHeld(FREE_ASPECT_KEY);
  // A capture ends the session, so whichever gesture starts one shuts the
  // others out until the session is over.
  const isCapturingRef = useRef(false);
  const generalSettings = useGeneralSettings();
  const captureOnDraw = generalSettings?.captureScreenshotOnDraw ?? false;
  const isIdle = recordingStatus === "idle";
  useScreenshotShortcut(!isScreenshotWindow);

  const activeMonitor = isScreenshotWindow
    ? isScreenshotCapture
      ? screenshotMonitor
      : null
    : recordingMode === "region"
      ? selectedMonitor
      : null;

  const { beginGesture, finishGesture } = useRegionGestureVisibility(
    isDragging || resizeDirection !== undefined,
  );

  const persistDraft = useCallback((): Region => {
    const persisted = snapRegion(draft);
    if (!isScreenshotSurface) setRegion(persisted);

    return persisted;
  }, [draft, isScreenshotSurface, setRegion]);

  useEffect(() => {
    if (screenshotMonitorId === null) return;

    let disposed = false;
    void listMonitors()
      .then((monitors) => {
        if (disposed) return;
        setScreenshotMonitor(
          monitors.find((monitor) => monitor.id === screenshotMonitorId) ??
            null,
        );
      })
      .catch((error: unknown) => {
        console.error("Could not resolve the screenshot monitor", error);
      });
    return () => {
      disposed = true;
    };
  }, []);

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
    if (!isScreenshotSurface) return;

    const cancel = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      // Escape closes only the borrowed screenshot overlay. The ruler was
      // already open before this session and remains available underneath it.
      void endScreenshotCapture();
    };

    window.addEventListener("keydown", cancel);

    return () => {
      window.removeEventListener("keydown", cancel);
    };
  }, [isScreenshotSurface]);

  useEffect(() => {
    void setRecordingControlsOpacity(isScreenshotCapture ? 0 : 1);
  }, [isScreenshotCapture]);

  useEffect(() => {
    if (!activeMonitor) {
      if (isScreenshotWindow) return;
      void setRegionSelectorPassthrough(true);
      void hideRegionSelector();
      return;
    }

    const fitted = fitRegion(
      region,
      activeMonitor.size.width,
      activeMonitor.size.height,
    );
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
    if (isScreenshotSurface) return;
    void showRegionSelector(activeMonitor);
  }, [activeMonitor, isScreenshotSurface, region, setRegion]);

  useEffect(() => {
    if (!activeMonitor) return;

    if (!isScreenshotWindow) void setRegionSelectorPassthrough(!isIdle);
    if (!isIdle || captureRequest === 0) return;

    let disposed = false;
    let metadata: { height: number; width: number } | undefined;
    let pixels: ArrayBuffer | undefined;
    const commit = () => {
      if (!disposed && metadata && pixels) {
        setScreenshot({ ...metadata, pixels });
      }
    };
    const channel = new Channel<ArrayBuffer>();
    channel.onmessage = (message) => {
      pixels = message;
      commit();
    };
    // Keep the prior complete snapshot installed until both parts of its
    // replacement arrive. The first magnifier simply appears when this first
    // request completes; opening the recording UI does no capture work.
    void takeMonitorScreenshot(activeMonitor.id, channel)
      .then((result) => {
        metadata = result;
        commit();
      })
      .catch((reason: unknown) => {
        console.error("Could not load the region magnifier image", reason);
      });
    return () => {
      disposed = true;
    };
  }, [activeMonitor, captureRequest, isIdle]);

  // A screenshot session starts without a region to manipulate or capture.
  const regionPlaced = hasRegion(draft);

  const center = () => {
    if (!activeMonitor || !regionPlaced) return;
    const centered = {
      ...draft,
      position: {
        x: wholePixel((activeMonitor.size.width - draft.size.width) / 2),
        y: wholePixel((activeMonitor.size.height - draft.size.height) / 2),
      },
    };
    setDraft(centered);
    if (!isScreenshotSurface) setRegion(centered);
  };

  const finish = useCallback(() => {
    if (!activeMonitor || !isScreenshotSurface || isCapturingRef.current)
      return;
    isCapturingRef.current = true;
    captureScreenshotRegion(
      screenshotDestination,
      activeMonitor.id,
      persistDraft(),
    );
  }, [activeMonitor, isScreenshotSurface, persistDraft]);

  // Screenshot capture keeps its toolbar while recording controls are hidden.
  const showActions =
    isScreenshotSurface &&
    !resizeDirection &&
    !isDragging &&
    !isDrawing &&
    !captureOnDraw;
  const canFinish = showActions && regionPlaced;

  useEffect(() => {
    if (!canFinish) return;

    const finishOnEnter = (event: KeyboardEvent) => {
      if (event.key !== "Enter" || event.repeat || event.isComposing) return;
      event.preventDefault();
      event.stopImmediatePropagation();
      finish();
    };

    window.addEventListener("keydown", finishOnEnter, true);
    return () => {
      window.removeEventListener("keydown", finishOnEnter, true);
    };
  }, [canFinish, finish]);

  if (!activeMonitor) return null;

  return (
    <main
      className={cn(
        "relative h-screen w-screen overflow-hidden select-none",
        resizeDirection && "cursor-none [&_*]:cursor-none!",
      )}
    >
      <RegionShade region={draft} />

      <RegionDrawingSurface
        aspect={freeAspect ? undefined : screenshotAspect}
        bounds={activeMonitor.size}
        current={draft}
        isEditing={isScreenshotSurface && isIdle}
        onChange={setDraft}
        onDrawingChange={setIsDrawing}
        onFinish={(nextRegion) => {
          // Releasing the region is the whole gesture when instant capture is
          // on: the region just drawn goes straight to the shot, since `draft`
          // may not have re-rendered with it yet.
          if (!captureOnDraw || isCapturingRef.current) return;
          isCapturingRef.current = true;
          captureScreenshotRegion(
            screenshotDestination,
            activeMonitor.id,
            snapRegion(nextRegion),
          );
        }}
      />

      <RegionTransformFrame
        aspectRatio={
          (isScreenshotSurface ? screenshotAspect : regionAspectRatio) || false
        }
        freeAspect={freeAspect}
        onChange={setDraft}
        onDraggingChange={setIsDragging}
        onGestureBegin={() => {
          if (!isScreenshotSurface) beginGesture();
        }}
        onGestureFinish={() => {
          if (!isScreenshotSurface) finishGesture();
        }}
        onPersist={persistDraft}
        onResizeDirectionChange={(direction) => {
          setResizeDirection(direction);
          if (direction) setCaptureRequest((current) => current + 1);
        }}
        region={draft}
        showHandles={!isScreenshotSurface}
        visible={isIdle && regionPlaced}
      />

      <ScreenshotRegionControls
        onAspectChange={setScreenshotAspect}
        onCenter={center}
        onFinish={finish}
        onSizeChange={(size) => {
          setDraft((current) => ({ ...current, size }));
        }}
        region={draft}
        regionPlaced={regionPlaced}
        visible={showActions}
      />

      {screenshot ? (
        <Magnifier
          regionRect={{
            height: draft.size.height,
            width: draft.size.width,
            x: draft.position.x,
            y: draft.position.y,
          }}
          resizeDirection={resizeDirection}
          screenshot={screenshot}
        />
      ) : null}
    </main>
  );
}
