// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { mouseControlFromButton } from "./hotkey";

const HOLD_DURATION_MS = 500;

/** Auxiliary buttons must survive a full hold/release before being accepted. */
export function useMouseControlCapture(
  enabled: boolean,
  onCapture: (value: string) => void,
  onRejected: () => void,
) {
  const [progress, setProgress] = useState<number | null>(null);
  const callbacksRef = useRef({ onCapture, onRejected });
  useEffect(() => {
    callbacksRef.current = { onCapture, onRejected };
  });

  useEffect(() => {
    if (!enabled) return;
    let frame: number | undefined;
    let held: { control: string; startedAt: number } | null = null;
    const reset = () => {
      if (frame !== undefined) cancelAnimationFrame(frame);
      frame = undefined;
      held = null;
      setProgress(null);
    };
    const tick = () => {
      if (!held) return;
      const elapsed = performance.now() - held.startedAt;
      setProgress(Math.min(100, (elapsed / HOLD_DURATION_MS) * 100));
      if (elapsed < HOLD_DURATION_MS) frame = requestAnimationFrame(tick);
    };
    const down = (event: PointerEvent) => {
      const control = mouseControlFromButton(event.button);
      if (!control) return;
      event.preventDefault();
      event.stopPropagation();
      reset();
      held = { control, startedAt: performance.now() };
      setProgress(0);
      frame = requestAnimationFrame(tick);
    };
    const up = (event: PointerEvent) => {
      const control = mouseControlFromButton(event.button);
      if (!control) return;
      event.preventDefault();
      event.stopPropagation();
      const confirmed =
        held?.control === control &&
        performance.now() - held.startedAt >= HOLD_DURATION_MS;
      reset();
      if (confirmed) callbacksRef.current.onCapture(control);
      else callbacksRef.current.onRejected();
    };
    const preventAuxClick = (event: MouseEvent) => {
      if (mouseControlFromButton(event.button)) event.preventDefault();
    };
    window.addEventListener("pointerdown", down, true);
    window.addEventListener("pointerup", up, true);
    window.addEventListener("pointercancel", reset, true);
    window.addEventListener("auxclick", preventAuxClick, true);
    return () => {
      window.removeEventListener("pointerdown", down, true);
      window.removeEventListener("pointerup", up, true);
      window.removeEventListener("pointercancel", reset, true);
      window.removeEventListener("auxclick", preventAuxClick, true);
      reset();
    };
  }, [enabled]);
  return enabled ? progress : null;
}
