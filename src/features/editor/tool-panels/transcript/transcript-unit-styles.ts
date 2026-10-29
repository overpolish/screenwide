// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../../lib/styling";

import { TranscriptUnit } from "./transcript-units";

import type { TranscriptMark } from "./use-transcript-marks";

/** How a word stands beyond itself: what it is marked as, and whether a
 * Remove is being previewed, on it or elsewhere. */
export type UnitLook = {
  /** A Remove that would cut this word is hovered or focused. */
  isPreviewed: boolean;
  /** Some Remove is hovered or focused, so only what it would cut shows
   * red. */
  isPreviewing: boolean;
  mark: TranscriptMark | undefined;
};

// Words worth a look take the warning colour: those the transcriber was
// unsure of, and fillers to cut. Profanity takes the error colour, except
// while a Remove is previewed, when only what it would cut is red. Words
// corrected by hand take a quiet underline. A choice takes the selected
// surface, the text unchanged. The word being said takes the strong fill,
// as the caption being said does in the Captions mode; a choice or a
// preview over it keeps its own look.
export const unitClass = (
  unit: TranscriptUnit,
  isChosen: boolean,
  { isPreviewed, isPreviewing, mark }: UnitLook,
) =>
  cn(
    !isPreviewed && "not-aria-selected:data-current:bg-fill-strong",
    unit.removed
      ? "text-fg-tertiary line-through"
      : cn(
          (unit.unsure || mark === "filler") && "text-warning",
          mark === "profanity" && !isPreviewing && "text-error",
          unit.corrected &&
            "underline decoration-fg-tertiary underline-offset-4",
        ),
    isChosen && "bg-selected",
    isPreviewed && "bg-error/25 text-error",
  );

/** The letters a search found inside a word: a warning tint behind just
 * them, in the body colour so an unsure word stays readable over it. A word
 * that is chosen keeps the choice's surface whole instead. */
export const FOUND_TEXT_CLASS = "bg-warning/35 text-fg";

// A pause reads as a compact, rounded chip rather than a word: tertiary
// text on a subtle fill, struck through once it is cut, on the strong fill
// while it is the silence playing, with the same selected surface a word
// takes. It wraps to the next line whole, as a word does, never split in
// two. While Remove pauses is hovered, the pauses it would cut turn the
// error colour, as a destructive action marks what it takes, and that
// colour stands even over the silence playing.
export const pauseClass = (
  unit: TranscriptUnit,
  isChosen: boolean,
  isPreviewed = false,
) =>
  cn(
    "rounded-full bg-fill px-inset-sm whitespace-nowrap text-fg-tertiary",
    unit.removed && "line-through",
    isChosen && "bg-selected text-fg",
    isPreviewed
      ? "bg-error/25 text-error"
      : "not-aria-selected:data-current:bg-fill-strong",
  );
