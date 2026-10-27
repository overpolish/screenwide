// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef, useState } from "react";

import { wordEdge } from "../model/trims";
import { type Editor } from "../model/use-editor";

import { type Stretch } from "./range-cut";
import { useAltKey } from "./use-alt-key";

// The scissors work anywhere on the strip: with the Cut tool on beside the
// play button, or while ⌥ is held. A blade follows the pointer, so what a
// press will do shows before it is made; dragged, it
// draws a stretch, over springs and beats alike, which is chosen when let
// go and cut with Delete or from the island. A spring cut partway is the
// common case: the dull middle of a pause, its first seconds. Its edges sit
// in the gaps between words, as the cut will.
//
// A pause's end handle keeps ⌥ for its own trim, so a press there is left
// alone.

/** The shortest stretch, in seconds, a drag chooses; less is a slip. */
const LEAST_S = 0.1;

export function useRangeCut({
  editor,
  fractionOf,
  secondsAt,
}: {
  editor: Editor;
  fractionOf: (event: React.PointerEvent) => number;
  secondsAt: (event: React.PointerEvent) => number;
}) {
  // The scissors are in hand while ⌥ is held, or while the Cut tool is on.
  const isCutting = useAltKey() || editor.isCutting;
  const fromRef = useRef<number | null>(null);
  // Set by a drag, so the click that ends it does not choose what is under
  // the pointer as well.
  const wasDrawnRef = useRef(false);
  const [draft, setDraft] = useState<Stretch | null>(null);
  const [bladeX, setBladeX] = useState<number | null>(null);
  const voice = editor.isVoiced ? editor.recording : null;

  const stretchTo = (event: React.PointerEvent): Stretch => {
    const from = fromRef.current ?? 0;
    const to = secondsAt(event);
    return {
      end: wordEdge(Math.max(from, to), voice),
      start: wordEdge(Math.min(from, to), voice),
    };
  };

  return {
    bladeX: isCutting ? bladeX : null,
    draft,
    handlers: {
      onClickCapture: (event: React.MouseEvent) => {
        if (!wasDrawnRef.current) return;
        wasDrawnRef.current = false;
        event.stopPropagation();
      },
      onPointerDownCapture: (event: React.PointerEvent<HTMLElement>) => {
        const target = event.target as Element;
        const isScissors = event.altKey || editor.isCutting;
        if (!isScissors || target.closest("[data-pause-end]")) return;
        event.stopPropagation();
        event.currentTarget.setPointerCapture(event.pointerId);
        fromRef.current = secondsAt(event);
        wasDrawnRef.current = true;
        setDraft(stretchTo(event));
      },
      onPointerLeave: () => {
        setBladeX(null);
      },
      onPointerMoveCapture: (event: React.PointerEvent) => {
        setBladeX(fractionOf(event));
        if (fromRef.current === null) return;
        event.stopPropagation();
        setDraft(stretchTo(event));
      },
      onPointerUpCapture: (event: React.PointerEvent) => {
        if (fromRef.current === null) return;
        event.stopPropagation();
        const stretch = stretchTo(event);
        fromRef.current = null;
        setDraft(null);
        if (stretch.end - stretch.start >= LEAST_S)
          editor.select({ kind: "range", ...stretch });
      },
    },
  };
}
