// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Scissors } from "lucide-react";
import { useRef } from "react";

import { cn } from "../../../../lib/styling";

// A chosen lull's end. Dragged, it retimes the lull: the spring stretches or
// tightens and every frame stays. Dragged with ⌥ (Alt), it trims: what is
// dragged past is cut, with a seam to put it back, and the rest plays as
// recorded. What the drag does is fixed as it is pressed.

export function SpringEnd({
  isAltHeld,
  onDrag,
}: {
  /** Whether ⌥ is held, which shows the scissors before the press. */
  isAltHeld: boolean;
  /** Told where the pointer is as a share of the lull's drawn width, and
   * whether the drag trims. */
  onDrag: (share: number, isTrim: boolean) => void;
}) {
  const isTrimRef = useRef<boolean | null>(null);
  return (
    <span
      aria-hidden
      className={cn(
        "absolute -end-2 top-1/2 flex h-7 w-4 -translate-y-1/2 touch-none items-center justify-center",
        isAltHeld ? "cursor-col-resize" : "cursor-ew-resize",
      )}
      onDoubleClick={(event) => {
        event.stopPropagation();
      }}
      onPointerDown={(event) => {
        event.stopPropagation();
        event.currentTarget.setPointerCapture(event.pointerId);
        isTrimRef.current = event.altKey;
      }}
      onPointerMove={(event) => {
        const isTrim = isTrimRef.current;
        const box = event.currentTarget.parentElement?.getBoundingClientRect();
        if (isTrim === null || !box || box.width === 0) return;
        onDrag(Math.max(0, event.clientX - box.left) / box.width, isTrim);
      }}
      onPointerUp={() => {
        isTrimRef.current = null;
      }}
    >
      {isAltHeld ? (
        <span className="flex size-5 items-center justify-center rounded-full bg-primary text-on-primary ring-2 ring-recess">
          <Scissors className="size-icon-xs" />
        </span>
      ) : (
        <span className="h-5 w-1.5 rounded-full bg-primary ring-2 ring-recess" />
      )}
    </span>
  );
}
