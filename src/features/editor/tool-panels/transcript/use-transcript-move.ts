// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useLayoutEffect, useRef } from "react";

import { TranscriptMove } from "../tool-panel-transcript";

import { movedSelection } from "./transcript-moved-selection";
import { movedFootage, TranscriptOrder } from "./transcript-playback-order";
import {
  spanOf,
  TranscriptUnit,
  WordRange,
  WordSelection,
} from "./transcript-units";

/** One move made here: the units it carried, and where their words were
 * chosen before and after it. */
type Move = {
  carried: TranscriptUnit[];
  from: WordRange | null;
  to: WordRange | null;
};

/** Where to choose carried words once the transcript is laid out again. */
type Landing = {
  carried: TranscriptUnit[];
  near: number;
  onFound: (found: WordRange | null) => void;
};

const sameSpan = (a: WordRange | null, b: WordRange | null) => {
  if (!a || !b) return false;
  const [x, y] = [spanOf(a), spanOf(b)];
  return x.first === y.first && x.last === y.last;
};

/**
 * What a drop of chosen words does: hands `onMove` the footage they play
 * and where it lands (`movedFootage`), and once the transcript is laid out
 * again, chooses them where they landed. Undo and redo of a move made here
 * choose the words where they go back to, as a text editor does, while they
 * are still what is chosen.
 */
export function useTranscriptMove({
  onMove,
  onRedo,
  onUndo,
  order,
  selection,
  setSelection,
}: {
  onMove: (move: TranscriptMove) => void;
  onRedo: () => void;
  onUndo: () => void;
  order: TranscriptOrder;
  selection: WordRange | null;
  setSelection: (selection: WordRange | null) => void;
}) {
  const { units } = order;
  const movesRef = useRef<{ done: Move[]; undone: Move[] }>({
    done: [],
    undone: [],
  });
  // Read when the units next change, which is when the edit has landed:
  // before they are drawn, so no frame shows the old indices choosing other
  // words, and no key acts on them.
  const landingRef = useRef<Landing | null>(null);
  useLayoutEffect(() => {
    const landing = landingRef.current;
    if (!landing) return;
    landingRef.current = null;
    const found = movedSelection(units, landing.carried, landing.near);
    landing.onFound(found);
    setSelection(found);
  }, [setSelection, units]);

  const move = (span: WordSelection, before: number) => {
    const footage = movedFootage(order, span, before);
    if (!footage) return;
    const count = span.last - span.first + 1;
    const carried = units.slice(span.first, span.last + 1);
    const made: Move = {
      carried,
      from: movedSelection(units, carried, span.first),
      to: null,
    };
    movesRef.current = { done: [...movesRef.current.done, made], undone: [] };
    landingRef.current = {
      carried,
      near: before > span.last ? before - count : before,
      onFound: (found) => {
        made.to = found;
      },
    };
    onMove(footage);
  };

  return {
    move,
    redo: () => {
      const { done, undone } = movesRef.current;
      const last = undone[undone.length - 1] as Move | undefined;
      if (last?.to && sameSpan(selection, last.from)) {
        landingRef.current = {
          carried: last.carried,
          near: spanOf(last.to).first,
          onFound: (found) => {
            last.to = found;
          },
        };
        movesRef.current = {
          done: [...done, last],
          undone: undone.slice(0, -1),
        };
      }
      onRedo();
    },
    undo: () => {
      const { done, undone } = movesRef.current;
      const last = done[done.length - 1] as Move | undefined;
      if (last?.from && sameSpan(selection, last.to)) {
        landingRef.current = {
          carried: last.carried,
          near: spanOf(last.from).first,
          onFound: (found) => {
            last.from = found;
          },
        };
        movesRef.current = {
          done: done.slice(0, -1),
          undone: [...undone, last],
        };
      }
      onUndo();
    },
  };
}
