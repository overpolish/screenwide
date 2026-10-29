// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type PointerEvent, useRef, useState } from "react";

/** The words a Command-drag reaches, from `first` to `last`, and whether
 * it hides them from the captions or shows them. */
export type CaptionPaint = { first: number; hide: boolean; last: number };

const wordIndexOf = (target: EventTarget | Element | null) => {
  const index =
    target instanceof Element
      ? target.closest<HTMLElement>("[data-word]")?.dataset.word
      : undefined;
  return index === undefined ? null : Number(index);
};

/**
 * Command-click hides a word from the captions or shows it again, and
 * Command-drag does the same to every word it reaches, each left as the
 * first one was made: the gesture that removes and restores words when
 * editing the transcript, here taking them out of the captions. The words
 * show as they will be left while the drag lasts (`painted`), and are made
 * so together when it ends, as one step of the edit history.
 */
export function useCaptionPaint({
  isEnabled,
  isHidden,
  onPaint,
}: {
  isEnabled: boolean;
  isHidden: (word: number) => boolean;
  onPaint: (paint: CaptionPaint) => void;
}) {
  const [paint, setPaint] = useState<{
    anchor: number;
    focus: number;
    hide: boolean;
  } | null>(null);
  const paintRef = useRef(onPaint);
  paintRef.current = onPaint;

  const onPointerDown = (event: PointerEvent<HTMLElement>) => {
    if (!isEnabled || event.button !== 0) return;
    if (!event.metaKey && !event.ctrlKey) return;
    const anchor = wordIndexOf(event.target);
    if (anchor === null) return;
    // No text is chosen and no focus moves while words are painted.
    event.preventDefault();
    const hide = !isHidden(anchor);
    let reach = anchor;
    setPaint({ anchor, focus: anchor, hide });
    // Followed on the window, so the drag keeps going past the text's edge.
    const onMove = (move: globalThis.PointerEvent) => {
      const reached = wordIndexOf(
        document.elementFromPoint(move.clientX, move.clientY),
      );
      if (reached === null || reached === reach) return;
      reach = reached;
      setPaint({ anchor, focus: reach, hide });
    };
    const onUp = () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      setPaint(null);
      paintRef.current({
        first: Math.min(anchor, reach),
        hide,
        last: Math.max(anchor, reach),
      });
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  };

  const painted: CaptionPaint | null = paint
    ? {
        first: Math.min(paint.anchor, paint.focus),
        hide: paint.hide,
        last: Math.max(paint.anchor, paint.focus),
      }
    : null;
  return { onPointerDown, painted };
}
