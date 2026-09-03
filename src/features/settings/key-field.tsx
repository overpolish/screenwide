// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { Button } from "../../components/base/button/button";
import { CircularProgress } from "../../components/base/circular-progress/circular-progress";
import { Keyboard } from "../../components/base/keyboard/keyboard";

import { beginShortcutCapture, endShortcutCapture } from "./api";

const MOUSE_HOLD_DURATION_MS = 500;

const keyName = (code: string) => {
  if (code === "MouseMiddle") return "Middle click";
  if (code === "MouseBack") return "Mouse Back";
  if (code === "MouseForward") return "Mouse Forward";
  if (code.startsWith("Key")) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  if (code.startsWith("Meta"))
    return navigator.userAgent.includes("Mac") ? "⌘" : "Win";
  if (code.startsWith("Control"))
    return navigator.userAgent.includes("Mac") ? "⌃" : "Ctrl";
  if (code.startsWith("Alt"))
    return navigator.userAgent.includes("Mac") ? "⌥" : "Alt";
  if (code.startsWith("Shift")) return "⇧";
  return code.replace("Arrow", "").replace("Numpad", "Num ");
};

export function KeyField({
  ariaLabel,
  isDisabled,
  onChange,
  value,
}: {
  ariaLabel: string;
  isDisabled: boolean;
  onChange: (key: string) => void;
  value: string;
}) {
  const [listening, setListening] = useState(false);
  const [mouseProgress, setMouseProgress] = useState<number | null>(null);
  const [unsupported, setUnsupported] = useState(false);

  useEffect(() => {
    if (!listening) return;
    let animationFrame: number | null = null;
    let mouseDown: { control: string; startedAt: number } | null = null;
    const resetMouseTest = () => {
      if (animationFrame !== null) cancelAnimationFrame(animationFrame);
      animationFrame = null;
      mouseDown = null;
      setMouseProgress(null);
    };
    const onKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();
      if (event.repeat || event.code === "Unidentified") return;
      resetMouseTest();
      onChange(event.code);
      setListening(false);
      void endShortcutCapture();
    };
    const mouseControl = (button: number) =>
      ({ 1: "MouseMiddle", 3: "MouseBack", 4: "MouseForward" })[button];
    const onPointerDown = (event: PointerEvent) => {
      const control = mouseControl(event.button);
      if (!control) return;
      event.preventDefault();
      event.stopPropagation();
      setUnsupported(false);
      mouseDown = { control, startedAt: performance.now() };
      setMouseProgress(0);
      const updateProgress = () => {
        if (!mouseDown) return;
        const elapsed = performance.now() - mouseDown.startedAt;
        setMouseProgress(
          Math.min(100, (elapsed / MOUSE_HOLD_DURATION_MS) * 100),
        );
        if (elapsed < MOUSE_HOLD_DURATION_MS) {
          animationFrame = requestAnimationFrame(updateProgress);
        }
      };
      animationFrame = requestAnimationFrame(updateProgress);
    };
    const onPointerUp = (event: PointerEvent) => {
      const control = mouseControl(event.button);
      if (!control) return;
      event.preventDefault();
      event.stopPropagation();
      const heldFor =
        mouseDown?.control === control
          ? performance.now() - mouseDown.startedAt
          : 0;
      resetMouseTest();
      if (heldFor < MOUSE_HOLD_DURATION_MS) {
        setUnsupported(true);
        return;
      }
      onChange(control);
      setListening(false);
      void endShortcutCapture();
    };
    window.addEventListener("keydown", onKeyDown, true);
    window.addEventListener("pointerdown", onPointerDown, true);
    window.addEventListener("pointercancel", resetMouseTest, true);
    window.addEventListener("pointerup", onPointerUp, true);
    return () => {
      window.removeEventListener("keydown", onKeyDown, true);
      window.removeEventListener("pointerdown", onPointerDown, true);
      window.removeEventListener("pointercancel", resetMouseTest, true);
      window.removeEventListener("pointerup", onPointerUp, true);
      resetMouseTest();
    };
  }, [listening, onChange]);

  return (
    <div className="flex flex-col items-end gap-1">
      <Button
        aria-label={ariaLabel}
        className="min-w-24 justify-center"
        isDisabled={isDisabled}
        onPress={() => {
          setUnsupported(false);
          void beginShortcutCapture().then(() => {
            setListening(true);
          });
        }}
        size="compact"
        variant="ghost"
      >
        {listening ? (
          mouseProgress === null ? (
            <span className="text-xs text-muted">
              Press key or hold button…
            </span>
          ) : (
            <span className="flex items-center gap-1.5 text-xs text-muted">
              <CircularProgress
                aria-label="Testing modifier hold"
                size="compact"
                value={mouseProgress}
              />
              Keep holding…
            </span>
          )
        ) : (
          <Keyboard size="sm">{keyName(value)}</Keyboard>
        )}
      </Button>
      {unsupported ? (
        <span className="max-w-40 whitespace-normal text-right text-xs text-error">
          This modifier is not supported.
        </span>
      ) : null}
    </div>
  );
}
