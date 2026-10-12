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
// corrected by hand take a quiet underline. A choice takes the accent tint
// and keeps its text in the body colour so it stays readable over the tint.
// The word being said takes the accent's text colour; a choice or a preview
// over it keeps its own look.
export const unitClass = (
  unit: TranscriptUnit,
  isChosen: boolean,
  { isPreviewed, isPreviewing, mark }: UnitLook,
) =>
  cn(
    !isPreviewed && "not-aria-selected:data-current:text-accent-heading",
    unit.removed
      ? "text-content-fg-tertiary line-through"
      : cn(
          (unit.unsure || mark === "filler") && "text-warning",
          mark === "profanity" && !isPreviewing && "text-error",
          unit.corrected &&
            "underline decoration-content-fg-tertiary underline-offset-4",
          isChosen && "text-content-fg",
        ),
    isChosen && "bg-primary-tint",
    isPreviewed && "bg-error/25 text-error",
  );

/** The letters a search found inside a word: a warning tint behind just
 * them, in the body colour so an unsure word stays readable over it. A word
 * that is chosen keeps the choice's tint whole instead. */
export const FOUND_TEXT_CLASS = "bg-warning/35 text-content-fg";

// A pause reads as a compact, rounded chip rather than a word: tertiary
// text on a subtle fill, struck through once it is cut, in the accent while
// it is the silence playing, with the same selection tint a word takes. It
// wraps to the next line whole, as a word does, never split in two. While
// Remove silences is hovered, the pauses it would cut turn the error colour,
// as a destructive action marks what it takes, and that colour stands even
// over the silence playing.
export const pauseClass = (
  unit: TranscriptUnit,
  isChosen: boolean,
  isPreviewed = false,
) =>
  cn(
    "rounded-full bg-fill px-control-inset whitespace-nowrap text-content-fg-tertiary",
    unit.removed && "line-through",
    isChosen && "bg-primary-tint text-content-fg",
    isPreviewed
      ? "bg-error/25 text-error"
      : "not-aria-selected:data-current:text-accent-heading",
  );
