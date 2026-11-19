// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { animate } from "motion/react";
import { type RefObject, useLayoutEffect, useRef } from "react";

import { motionDurations, motionEasings } from "../../../lib/motion";
import { type PopupMenuAnchor } from "../../popup-panel/use-popup-menu";
import { type ScriptText } from "../script-words";

import { wordAt } from "./cue-anchors";
import { CueMark } from "./cue-mark";

/** A cue as the text shows it: its mark, a tiny copy of its picture at
 * `src`, beside the word starting `at`. */
export type MarkedCue = { at: number; id: string; src: string; title: string };

const px = (value: number) => `${String(value)}px`;

// Each cue's mark at the column's end, level with its word's line. The marks
// stand in a layer of their own over the words rather than among them, so a
// cue moved to another word keeps its mark, which slides to the new line.
// Where each line is can only be read once the words are laid out, so the
// marks are put there after each render, before it is painted. The mark
// being dragged follows the pointer instead, and slides onto its line once
// let go.
export function CueMarks({
  dragged,
  marks,
  onMarkPress,
  script,
  wordsRef,
}: {
  /** The cue being dragged, and how far down the screen its pointer is. */
  dragged: { clientY: number; id: string } | null;
  marks: readonly MarkedCue[];
  onMarkPress: (
    cue: MarkedCue,
    press: { clientX: number; clientY: number },
    anchor: PopupMenuAnchor,
  ) => void;
  script: ScriptText;
  /** Each word's element, by its index, as the words draw them. */
  wordsRef: RefObject<Map<number, HTMLSpanElement>>;
}) {
  const elementsRef = useRef(new Map<string, HTMLElement>());
  // Where each mark was put last, by cue: one not there yet goes straight
  // to its line, one there already slides.
  const middlesRef = useRef(new Map<string, number>());
  const slidesRef = useRef(new Map<string, { stop: () => void }>());

  // Edits, rewraps and moves all shift lines, so this runs after every
  // render; a mark whose line has not moved is left alone, and a cue gone
  // is forgotten, so one made again is put straight on its line.
  useLayoutEffect(() => {
    const middles = middlesRef.current;
    const slides = slidesRef.current;
    for (const id of middles.keys())
      if (!marks.some((mark) => mark.id === id)) middles.delete(id);
    for (const mark of marks) {
      const element = elementsRef.current.get(mark.id);
      const word = wordsRef.current.get(wordAt(script, mark.at));
      if (!element || !word) continue;
      // Measured from the column as it stands on screen, scrolled with it.
      const column = element.offsetParent?.getBoundingClientRect().top ?? 0;
      const middle =
        mark.id === dragged?.id
          ? dragged.clientY - column
          : word.offsetTop + word.offsetHeight / 2;
      const was = middles.get(mark.id);
      if (was === middle) continue;
      middles.set(mark.id, middle);
      slides.get(mark.id)?.stop();
      if (was === undefined || mark.id === dragged?.id) {
        element.style.top = px(middle);
        continue;
      }
      // From where it is drawn: Motion would otherwise start from where it
      // last took the mark, which the pointer has since moved it from.
      const from = getComputedStyle(element).top;
      slides.set(
        mark.id,
        animate(
          element,
          { top: [from, px(middle)] },
          { duration: motionDurations.rearrange, ease: motionEasings.out },
        ),
      );
    }
  });

  return marks.map((mark) => (
    <CueMark
      isDragged={mark.id === dragged?.id}
      key={mark.id}
      onPress={(press, anchor) => {
        onMarkPress(mark, press, anchor);
      }}
      ref={(element) => {
        if (element) elementsRef.current.set(mark.id, element);
        else elementsRef.current.delete(mark.id);
      }}
      src={mark.src}
      title={mark.title}
    />
  ));
}
