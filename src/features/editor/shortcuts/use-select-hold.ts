// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";

import { hostPlatform } from "../../../lib/platform";

import { ownsTextEditingKeys } from "./keyboard-target";
import { isSelectHoldKey, SelectHold, selectHold } from "./select-hold";

/** A press on the native picture, as `preview://native-pointer` reports it:
 * `down` as it goes down, with `took` when it took hold of something already
 * there, and `down: false` as it is let go. */
export type NativePointerEvent = { down: boolean; took: boolean };

export const NATIVE_POINTER_EVENT = "preview://native-pointer";

/**
 * Select held under ⌘ (Ctrl on Windows): whether the picture acts as Select
 * now. It holds only from a pen (`canHold`), and only when the modifier goes
 * down with nothing pressed, in the page or on the picture. A press under it
 * that takes hold of something calls `onTook`, for the editor to put Select
 * in hand for good, so letting go of the modifier leaves it there; with
 * nothing taken, letting go puts the pen back. Another key with the modifier
 * (⌘Z, ⌘D) and the window losing focus end the hold at once.
 */
export function useSelectHold({
  canHold,
  onTook,
}: {
  canHold: boolean;
  onTook: () => void;
}) {
  const [state, setState] = useState<SelectHold>("idle");
  const stateRef = useRef(state);
  const canHoldRef = useRef(canHold);
  canHoldRef.current = canHold;
  const onTookRef = useRef(onTook);
  onTookRef.current = onTook;
  useEffect(() => {
    let pagePressed = false;
    let picturePressed = false;
    const apply = (event: Parameters<typeof selectHold>[1]) => {
      const next = selectHold(stateRef.current, event);
      if (next === stateRef.current) return;
      stateRef.current = next;
      setState(next);
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (!isSelectHoldKey(event.key, hostPlatform)) {
        apply({ type: "otherKey" });
        return;
      }
      if (event.repeat) return;
      apply({
        canHold:
          canHoldRef.current &&
          !pagePressed &&
          !picturePressed &&
          !ownsTextEditingKeys(event.target),
        type: "modifierDown",
      });
    };
    const onKeyUp = (event: KeyboardEvent) => {
      if (isSelectHoldKey(event.key, hostPlatform))
        apply({ type: "modifierUp" });
    };
    const onBlur = () => {
      pagePressed = false;
      apply({ type: "blur" });
    };
    const onPointerDown = () => {
      pagePressed = true;
    };
    const onPointerUp = () => {
      pagePressed = false;
    };
    window.addEventListener("keydown", onKeyDown, true);
    window.addEventListener("keyup", onKeyUp, true);
    window.addEventListener("blur", onBlur);
    window.addEventListener("pointerdown", onPointerDown, true);
    window.addEventListener("pointerup", onPointerUp, true);
    window.addEventListener("pointercancel", onPointerUp, true);
    let unlisten: (() => void) | undefined;
    let disposed = false;
    void getCurrentWindow()
      .listen<NativePointerEvent>(NATIVE_POINTER_EVENT, ({ payload }) => {
        picturePressed = payload.down;
        if (payload.took && stateRef.current === "held") onTookRef.current();
      })
      .then((dispose) => {
        if (disposed) dispose();
        else unlisten = dispose;
      });
    return () => {
      disposed = true;
      unlisten?.();
      window.removeEventListener("keydown", onKeyDown, true);
      window.removeEventListener("keyup", onKeyUp, true);
      window.removeEventListener("blur", onBlur);
      window.removeEventListener("pointerdown", onPointerDown, true);
      window.removeEventListener("pointerup", onPointerUp, true);
      window.removeEventListener("pointercancel", onPointerUp, true);
    };
  }, []);
  return state === "held";
}
