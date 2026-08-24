// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Channel } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";

import { cn } from "../../lib/styling";
import { selectStatus, useRecordingStore } from "../recording-controls/store";
import {
  hideRegionSelector,
  setRecordingControlsOpacity,
  setRegionSelectorPassthrough,
  showRegionSelector,
  takeMonitorScreenshot,
} from "../recording-sources/api";
import { useRecordingSourceStore } from "../recording-sources/store";
import { Region } from "../recording-sources/types";
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
  revealScreenshotRegion,
} from "./screenshot-session";
import { ResizeDirection } from "./types";
import { useKeyHeld } from "./use-key-held";
import { useRegionGestureVisibility } from "./use-region-gesture-visibility";
import { useScreenshotShortcut } from "./use-screenshot-shortcut";

// Holding this ignores the linked ratio, so the region can be reshaped
// freely; the shape it ends up with becomes the new ratio.
const FREE_ASPECT_KEY = "Shift";

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
  const [draft, setDraft] = useState(region);
  const [seededForSession, setSeededForSession] = useState(isScreenshotCapture);
  if (seededForSession !== isScreenshotCapture) {
    // Reseed while rendering rather than in an effect: otherwise the session
    // would leave one painted frame of the recording marquee before the empty
    // draft landed - a visible flash when the overlay was already on screen.
    setSeededForSession(isScreenshotCapture);
    setDraft(isScreenshotCapture ? EMPTY_REGION : region);
  }
  const [screenshotAspect, setScreenshotAspect] = useState<number>();
  const [resizeDirection, setResizeDirection] = useState<ResizeDirection>();
  const [isDragging, setIsDragging] = useState(false);
  const [isDrawing, setIsDrawing] = useState(false);
  const [screenshot, setScreenshot] = useState<ArrayBuffer | null>(null);
  const freeAspect = useKeyHeld(FREE_ASPECT_KEY);
  // A capture ends the session, so whichever gesture starts one shuts the
  // others out until the session is over.
  const isCapturingRef = useRef(false);
  const generalSettings = useGeneralSettings();
  const captureOnDraw = generalSettings?.captureScreenshotOnDraw ?? false;
  const isIdle = recordingStatus === "idle";
  useScreenshotShortcut();

  const activeMonitor =
    recordingMode === "region" || isScreenshotCapture ? selectedMonitor : null;

  const { beginGesture, finishGesture } = useRegionGestureVisibility(
    isDragging || resizeDirection !== undefined,
  );

  const persistDraft = useCallback((): Region => {
    const persisted = snapRegion(draft);
    if (!isScreenshotCapture) setRegion(persisted);

    return persisted;
  }, [draft, isScreenshotCapture, setRegion]);

  useEffect(() => {
    // Cross-window storage updates replace the persisted region. A screenshot
    // session starts from nothing instead: the region is drawn for that shot
    // alone, and the persisted one comes back when the session ends.
    // eslint-disable-next-line @eslint-react/set-state-in-effect
    setDraft(isScreenshotCapture ? EMPTY_REGION : region);
  }, [isScreenshotCapture, region]);

  useEffect(() => {
    // Each session gets its one capture back.
    isCapturingRef.current = false;
    if (!isScreenshotCapture) return;

    const cancel = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      void endScreenshotCapture();
    };

    window.addEventListener("keydown", cancel);

    return () => {
      window.removeEventListener("keydown", cancel);
    };
  }, [isScreenshotCapture]);

  useEffect(() => {
    void setRecordingControlsOpacity(isScreenshotCapture ? 0 : 1);
  }, [isScreenshotCapture]);

  useEffect(() => {
    if (!activeMonitor) {
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
    setDraft(isScreenshotCapture ? EMPTY_REGION : fitted);
    if (
      !isScreenshotCapture &&
      JSON.stringify(fitted) !== JSON.stringify(region)
    ) {
      setRegion(fitted);
    }
    if (isScreenshotCapture) return revealScreenshotRegion(activeMonitor);
    void showRegionSelector(activeMonitor);
  }, [activeMonitor, isScreenshotCapture, region, setRegion]);

  useEffect(() => {
    if (!activeMonitor) return;

    void setRegionSelectorPassthrough(!isIdle);
    if (!isIdle) return;

    const channel = new Channel<ArrayBuffer>();
    channel.onmessage = setScreenshot;
    // Clear the prior monitor image before asynchronously capturing the next
    // one. The capture leaves Screenwide's windows out, so the overlay stays
    // exactly as it is while the image is taken.
    // eslint-disable-next-line @eslint-react/set-state-in-effect
    setScreenshot(null);
    void takeMonitorScreenshot(activeMonitor.id, channel);
  }, [activeMonitor, isIdle]);

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
    if (!isScreenshotCapture) setRegion(centered);
  };

  const finish = useCallback(() => {
    if (!activeMonitor || !isScreenshotCapture || isCapturingRef.current)
      return;
    isCapturingRef.current = true;
    captureScreenshotRegion(activeMonitor.id, persistDraft());
  }, [activeMonitor, isScreenshotCapture, persistDraft]);

  // Screenshot capture keeps its toolbar while recording controls are hidden.
  const showActions =
    isScreenshotCapture &&
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
        isEditing={isScreenshotCapture && isIdle}
        onChange={setDraft}
        onDrawingChange={setIsDrawing}
        onFinish={(nextRegion) => {
          // Releasing the region is the whole gesture when instant capture is
          // on: the region just drawn goes straight to the shot, since `draft`
          // may not have re-rendered with it yet.
          if (!captureOnDraw || isCapturingRef.current) return;
          isCapturingRef.current = true;
          captureScreenshotRegion(activeMonitor.id, snapRegion(nextRegion));
        }}
      />

      <RegionTransformFrame
        aspectRatio={
          (isScreenshotCapture ? screenshotAspect : regionAspectRatio) || false
        }
        freeAspect={freeAspect}
        onChange={setDraft}
        onDraggingChange={setIsDragging}
        onGestureBegin={() => {
          if (!isScreenshotCapture) beginGesture();
        }}
        onGestureFinish={() => {
          if (!isScreenshotCapture) finishGesture();
        }}
        onPersist={persistDraft}
        onResizeDirectionChange={setResizeDirection}
        region={draft}
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
          monitor={activeMonitor}
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
