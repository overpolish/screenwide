// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  PointerEvent as ReactPointerEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
  WheelEvent as ReactWheelEvent,
} from "react";

import { ownsTextEditingKeys } from "../keyboard-target";

import {
  fitTimelineViewport,
  panTimelineViewportByPixels,
  zoomTimelineViewportAt,
} from "./timeline-viewport";

const ZOOM_WHEEL_SENSITIVITY = 0.006;

export function useTimelineNavigation(resetKey: unknown) {
  const [viewport, setViewport] = useState(fitTimelineViewport);
  const areaRef = useRef<HTMLDivElement>(null);
  const panRef = useRef<{ pointerId: number; x: number } | null>(null);

  useEffect(() => {
    // Navigation is view state, so a new recording resets it while retained
    // duration changes from timeline edits leave the viewport untouched.
    // eslint-disable-next-line @eslint-react/set-state-in-effect
    setViewport(fitTimelineViewport());
  }, [resetKey]);

  const zoom = useCallback((factor: number, clientX?: number) => {
    const bounds = areaRef.current?.getBoundingClientRect();
    if (!bounds) return;
    setViewport((current) =>
      zoomTimelineViewportAt(current, {
        cursorX: clientX ?? bounds.left + bounds.width / 2,
        factor,
        rect: bounds,
      }),
    );
  }, []);

  const fit = useCallback(() => {
    setViewport(fitTimelineViewport());
  }, []);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (
        event.code !== "KeyZ" ||
        !event.shiftKey ||
        event.altKey ||
        event.ctrlKey ||
        event.metaKey ||
        event.isComposing ||
        event.repeat ||
        ownsTextEditingKeys(event.target)
      )
        return;
      event.preventDefault();
      fit();
    };
    window.addEventListener("keydown", onKeyDown, true);
    return () => {
      window.removeEventListener("keydown", onKeyDown, true);
    };
  }, [fit]);

  const onWheel = (event: ReactWheelEvent<HTMLElement>) => {
    const bounds = areaRef.current?.getBoundingClientRect();
    if (!bounds || event.clientX < bounds.left || event.clientX > bounds.right)
      return;
    const lineScale = event.deltaMode === 1 ? 16 : 1;
    const pageScale = event.deltaMode === 2 ? bounds.width : 1;
    const scale = event.deltaMode === 2 ? pageScale : lineScale;
    if (event.ctrlKey || event.metaKey) {
      event.preventDefault();
      zoom(
        Math.exp(-event.deltaY * scale * ZOOM_WHEEL_SENSITIVITY),
        event.clientX,
      );
      return;
    }
    const delta = event.shiftKey
      ? event.deltaY * scale
      : Math.abs(event.deltaX) > Math.abs(event.deltaY) * 1.2
        ? event.deltaX * scale
        : 0;
    if (delta === 0) return;
    event.preventDefault();
    setViewport((current) =>
      panTimelineViewportByPixels(current, -delta, bounds.width),
    );
  };

  const onPointerDown = (event: ReactPointerEvent<HTMLElement>) => {
    if (event.button !== 1) return;
    const bounds = areaRef.current?.getBoundingClientRect();
    if (!bounds || event.clientX < bounds.left || event.clientX > bounds.right)
      return;
    event.preventDefault();
    panRef.current = { pointerId: event.pointerId, x: event.clientX };
    event.currentTarget.setPointerCapture(event.pointerId);
  };

  const onPointerMove = (event: ReactPointerEvent<HTMLElement>) => {
    const active = panRef.current;
    const bounds = areaRef.current?.getBoundingClientRect();
    if (!active || active.pointerId !== event.pointerId || !bounds) return;
    const delta = event.clientX - active.x;
    active.x = event.clientX;
    setViewport((current) =>
      panTimelineViewportByPixels(current, delta, bounds.width),
    );
  };

  const onPointerEnd = (event: ReactPointerEvent<HTMLElement>) => {
    if (panRef.current?.pointerId !== event.pointerId) return;
    panRef.current = null;
    if (event.currentTarget.hasPointerCapture(event.pointerId))
      event.currentTarget.releasePointerCapture(event.pointerId);
  };

  return {
    areaRef,
    fit,
    interactionProps: {
      onAuxClick: (event: ReactPointerEvent<HTMLElement>) => {
        if (event.button === 1) event.preventDefault();
      },
      onPointerCancel: onPointerEnd,
      onPointerDown,
      onPointerMove,
      onPointerUp: onPointerEnd,
      onWheel,
    },
    viewport,
    zoom,
  };
}
