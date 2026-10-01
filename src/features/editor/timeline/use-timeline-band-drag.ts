// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

/**
 * The band's resize drag: a press on its top edge, followed until the pointer
 * lets go. Escape and the window losing focus put the band back where the
 * press found it.
 *
 * The pointer is followed on the window rather than through the handle's own
 * pointer capture: the handle is a few pixels tall and slides under the
 * pointer as the band grows, so tracking that depends on it staying the
 * capture target loses the drag the moment the edge moves past the pointer.
 */
export function useTimelineBandDrag({
  clamp,
  height,
  setHeight,
}: {
  clamp: (value: number) => number;
  /** The height the band stands at now, which is where a drag starts from. */
  height: number;
  setHeight: (height: number) => void;
}) {
  const dragRef = useRef<{ height: number; y: number } | null>(null);
  const cancel = () => {
    if (!dragRef.current) return;
    setHeight(dragRef.current.height);
    dragRef.current = null;
  };
  const cancelRef = useRef(cancel);
  cancelRef.current = cancel;
  useEffect(() => {
    const blur = () => {
      cancelRef.current();
    };
    const escape = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || !dragRef.current) return;
      event.preventDefault();
      event.stopImmediatePropagation();
      blur();
    };
    window.addEventListener("keydown", escape, true);
    window.addEventListener("blur", blur);
    return () => {
      window.removeEventListener("keydown", escape, true);
      window.removeEventListener("blur", blur);
    };
  }, []);
  const update = (y: number) => {
    if (dragRef.current)
      setHeight(clamp(dragRef.current.height + dragRef.current.y - y));
  };
  const updateRef = useRef(update);
  updateRef.current = update;
  const releaseRef = useRef<(() => void) | null>(null);
  const beginDrag = (y: number) => {
    releaseRef.current?.();
    dragRef.current = { height, y };
    const move = (event: PointerEvent) => {
      updateRef.current(event.clientY);
    };
    const finish = (event: PointerEvent) => {
      updateRef.current(event.clientY);
      dragRef.current = null;
      releaseRef.current?.();
    };
    const stop = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", finish);
      window.removeEventListener("pointercancel", abandon);
      releaseRef.current = null;
    };
    function abandon() {
      cancelRef.current();
      stop();
    }
    releaseRef.current = stop;
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", finish);
    window.addEventListener("pointercancel", abandon);
  };
  useEffect(() => () => releaseRef.current?.(), []);
  /** Forgets a drag without moving the band: for a refit that supersedes it. */
  const dropDrag = () => {
    dragRef.current = null;
  };
  return { beginDrag, dropDrag };
}
