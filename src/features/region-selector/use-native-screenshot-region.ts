// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

import { setScreenshotRegionOsc } from "../recording-sources/api";

import {
  emptyRegion,
  type NativePayload,
  type NativeScreenshotRegionOptions,
  nativeRegion,
  resizeDirections,
} from "./native-region-wire";

export function useNativeScreenshotRegion({
  aspect,
  allowDrawing = true,
  bounds,
  desktop = false,
  enabled,
  exclusionRect,
  onFinished,
  onGesture,
  onMonitorChange,
  onReconciled,
  onRegionChange,
  region,
  visible,
  showFrame = true,
  showHandles = true,
  inputEnabled = visible,
  monitorId,
  windowLabel,
}: NativeScreenshotRegionOptions) {
  const [available, setAvailable] = useState(false);
  const [layoutRevision, setLayoutRevision] = useState(0);
  const callbacksRef = useRef({
    onFinished,
    onGesture,
    onMonitorChange,
    onReconciled,
    onRegionChange,
  });
  const configRef = useRef({
    allowDrawing,
    aspect,
    bounds,
    desktop,
    exclusionRect,
    inputEnabled,
    monitorId,
    region,
    showFrame,
    showHandles,
    visible,
  });
  const handshakeRef = useRef(0);
  const nativeGestureRef = useRef(false);
  callbacksRef.current = {
    onFinished,
    onGesture,
    onMonitorChange,
    onReconciled,
    onRegionChange,
  };
  configRef.current = {
    allowDrawing,
    aspect,
    bounds,
    desktop,
    exclusionRect,
    inputEnabled,
    monitorId,
    region,
    showFrame,
    showHandles,
    visible,
  };

  useEffect(() => {
    const handshake = handshakeRef.current + 1;
    handshakeRef.current = handshake;
    if (!enabled || !windowLabel) {
      // eslint-disable-next-line @eslint-react/set-state-in-effect
      setAvailable(false);
      return;
    }
    let unlisten: (() => void) | undefined;
    let unlistenLayout: (() => void) | undefined;
    void (async () => {
      try {
        unlisten = await listen<NativePayload>(
          "screenshot-region-osc",
          ({ payload }) => {
            nativeGestureRef.current =
              payload.status === "changed" && payload.gesture !== null;
            if (payload.monitorId !== undefined)
              callbacksRef.current.onMonitorChange?.(payload.monitorId);
            const next = nativeRegion(payload.region);
            callbacksRef.current.onRegionChange(next ?? emptyRegion());
            if (payload.status === "layout") {
              nativeGestureRef.current = false;
              callbacksRef.current.onGesture({
                dragging: false,
                drawing: false,
                resizeDirection: undefined,
              });
              callbacksRef.current.onReconciled?.(next ?? emptyRegion());
              return;
            }
            if (payload.status === "changed") {
              const resizeDirection =
                payload.gesture && typeof payload.gesture === "object"
                  ? resizeDirections[payload.gesture.resizing.handle]
                  : undefined;
              callbacksRef.current.onGesture({
                dragging: payload.gesture === "moving",
                drawing: payload.gesture === "drawing",
                resizeDirection,
              });
            } else {
              callbacksRef.current.onGesture({
                dragging: false,
                drawing: false,
                resizeDirection: undefined,
              });
              if (payload.status === "finished" && payload.gesture && next)
                callbacksRef.current.onFinished(
                  next,
                  payload.gesture,
                  payload.monitorId,
                );
            }
          },
          { target: windowLabel },
        );
        unlistenLayout = await listen(
          "screenshot-region-desktop-layout",
          () => {
            if (configRef.current.desktop)
              setLayoutRevision((revision) => revision + 1);
          },
          { target: windowLabel },
        );
        if (handshake !== handshakeRef.current) {
          unlisten();
          unlistenLayout();
          return;
        }
        const c = configRef.current;
        const ok = await setScreenshotRegionOsc({
          allowDrawing: c.allowDrawing,
          aspect: c.aspect,
          bounds: c.bounds,
          desktop: c.desktop,
          exclusionRect: c.exclusionRect,
          inputEnabled: false,
          monitorId: c.monitorId,
          region: {
            height: c.region.size.height,
            width: c.region.size.width,
            x: c.region.position.x,
            y: c.region.position.y,
          },
          showFrame: c.showFrame,
          showHandles: c.showHandles,
          visible: c.visible,
          window: windowLabel,
        });
        if (handshake === handshakeRef.current) setAvailable(ok);
      } catch (error) {
        console.error("Could not attach the native Region OSC", error);
        if (handshake === handshakeRef.current) setAvailable(false);
      }
    })();
    return () => {
      handshakeRef.current += 1;
      unlisten?.();
      unlistenLayout?.();
      const c = configRef.current;
      void setScreenshotRegionOsc({
        allowDrawing: c.allowDrawing,
        aspect: c.aspect,
        bounds: c.bounds,
        desktop: c.desktop,
        inputEnabled: false,
        monitorId: c.monitorId,
        region: {
          height: c.region.size.height,
          width: c.region.size.width,
          x: c.region.position.x,
          y: c.region.position.y,
        },
        showFrame: c.showFrame,
        showHandles: c.showHandles,
        visible: false,
        window: windowLabel,
      });
    };
  }, [enabled, windowLabel]);

  useEffect(() => {
    if (!available || !windowLabel) return;
    // Native input has already updated and drawn this region. Let React mirror
    // the semantic state without echoing every pointer sample back over IPC;
    // the finished/cancelled event performs one final authoritative sync.
    if (nativeGestureRef.current && visible) return;
    const disable = async () => {
      try {
        await setScreenshotRegionOsc({
          allowDrawing,
          aspect,
          bounds,
          desktop,
          inputEnabled: false,
          monitorId,
          region: {
            height: region.size.height,
            width: region.size.width,
            x: region.position.x,
            y: region.position.y,
          },
          showFrame,
          showHandles,
          visible: false,
          window: windowLabel,
        });
      } finally {
        setAvailable(false);
      }
    };
    void setScreenshotRegionOsc({
      allowDrawing,
      aspect,
      bounds,
      desktop,
      exclusionRect,
      inputEnabled,
      monitorId,
      region: {
        height: region.size.height,
        width: region.size.width,
        x: region.position.x,
        y: region.position.y,
      },
      showFrame,
      showHandles,
      visible,
      window: windowLabel,
    })
      .then((ok) => {
        if (!ok) void disable();
      })
      .catch(() => void disable());
  }, [
    allowDrawing,
    aspect,
    available,
    bounds,
    desktop,
    exclusionRect,
    inputEnabled,
    layoutRevision,
    monitorId,
    region,
    showFrame,
    showHandles,
    visible,
    windowLabel,
  ]);
  return available;
}
