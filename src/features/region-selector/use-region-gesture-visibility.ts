// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useRef } from "react";

import {
  beginRegionSelectorGesture,
  finishRegionSelectorGesture,
} from "../recording-sources/api";

/** Keeps temporarily hidden recording controls paired with one region gesture. */
export function useRegionGestureVisibility(active: boolean) {
  const gestureActiveRef = useRef(false);

  const beginGesture = useCallback(() => {
    if (gestureActiveRef.current) return;
    gestureActiveRef.current = true;
    void beginRegionSelectorGesture();
  }, []);

  const finishGesture = useCallback(() => {
    if (!gestureActiveRef.current) return;
    gestureActiveRef.current = false;
    void finishRegionSelectorGesture();
  }, []);

  useEffect(() => {
    if (!active) return;

    window.addEventListener("blur", finishGesture);
    window.addEventListener("pointercancel", finishGesture, true);
    window.addEventListener("pointerup", finishGesture, true);
    return () => {
      window.removeEventListener("blur", finishGesture);
      window.removeEventListener("pointercancel", finishGesture, true);
      window.removeEventListener("pointerup", finishGesture, true);
    };
  }, [active, finishGesture]);

  useEffect(
    () => () => {
      if (gestureActiveRef.current) void finishRegionSelectorGesture();
    },
    [],
  );

  return { beginGesture, finishGesture };
}
