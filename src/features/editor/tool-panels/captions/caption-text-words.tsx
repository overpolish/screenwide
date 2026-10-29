// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { cn } from "../../../../lib/styling";

import type { CaptionTextWord } from "./caption-text";
import type { TranscriptMark } from "../transcript/use-transcript-marks";

// A caption's words and the spaces between take a quiet fill while the
// pointer is over any of them, and a stronger one while it is said. Words
// the captions leave out read faded and struck through, as cut words do in
// the transcript; fillers and profanity they keep take the warning and
// error colours the Transcript panel marks them in, words corrected by hand
// its quiet underline, and a caption's key word reads bold. The word the
// keys are on takes the focus ring while the text has focus.
const CAPTION_PART = "data-hovered:bg-fill data-current:bg-fill-strong";

export const captionWordClass = (
  word: CaptionTextWord,
  isMarked: boolean,
  mark: TranscriptMark | undefined,
) =>
  cn(
    "group-focus-visible:aria-selected:ring-2 group-focus-visible:aria-selected:ring-focus",
    isMarked && word.caption === null
      ? "text-fg-tertiary line-through"
      : cn(
          CAPTION_PART,
          mark === "filler" && "text-warning",
          mark === "profanity" && "text-error",
          word.corrected &&
            "underline decoration-fg-tertiary underline-offset-4",
          word.isKeyWord && "font-semibold",
        ),
  );

/** What sits before the word at `index`: nothing for a paragraph's first
 * (no `prior`), the mark of a break, or a space, carried in its caption's
 * colour where it runs on. Each is the word's gap (`data-gap`), which a
 * right click asks about. A break placed by hand is blue, one the rules
 * place faint. */
export const beforeCaptionWord = (
  word: CaptionTextWord,
  index: number,
  prior: CaptionTextWord | null,
): ReactNode => {
  if (word.gap === "split" || word.gap === "join")
    return (
      // A no-break space ties the mark to the words before it, so a wrapped
      // line ends on it rather than opening with it.
      <span aria-hidden className="text-info" data-gap={index}>
        {word.gap === "split" ? "\u00a0| " : "\u00a0\u203f "}
      </span>
    );
  if (prior === null) return null;
  if (word.gap === "break")
    return (
      <span aria-hidden className="text-fg-tertiary" data-gap={index}>
        {"\u00a0| "}
      </span>
    );
  const runsOn = word.caption !== null && word.caption === prior.caption;
  return (
    <span
      className={runsOn ? CAPTION_PART : undefined}
      data-caption={runsOn ? (word.caption ?? undefined) : undefined}
      data-gap={index}
    >
      {" "}
    </span>
  );
};
