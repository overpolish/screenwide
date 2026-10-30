// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../../lib/styling";
import { placed, type Span, type TimelineScale } from "../timeline-scale";

// One line said, a caption. Where it has room, its words spread out along
// it, each at the moment it is said; with less, it shows its text cut short,
// and with less still, nothing. Pressed, it opens the captions on it. The
// captions are one setting for the whole recording, so a line has no state
// of its own beyond whether they are hidden from the picture, when it is
// drawn as an outline, still there to press and show them again.

/** A line's least width, in px, for each letter of it to spread its words
 * out along it. */
const SPREAD_PX_PER_LETTER = 14;
/** The narrowest a line shows its text at, in px. */
const TEXT_PX = 90;

export type TimelineCaption = Span & {
  text: string;
  /** Each word and the recorded moment it is said at. */
  words: { at: number; word: string }[];
};

export type CaptionLineProps = {
  /** Whether the captions are hidden from the picture. */
  isHidden: boolean;
  line: TimelineCaption;
  onPress: () => void;
  scale: TimelineScale;
  /** The strip's width, in px. */
  width: number;
};

export function CaptionLine({
  isHidden,
  line,
  onPress,
  scale,
  width,
}: CaptionLineProps) {
  const from = scale.toX(line.start);
  const length = scale.toX(line.end) - from;
  const px = length * width;
  const isSpread = px > line.text.length * SPREAD_PX_PER_LETTER;
  return (
    <button
      aria-label={line.text}
      className={cn(
        "absolute top-0 flex h-4 cursor-default items-center overflow-hidden rounded-full px-1.5 text-caption whitespace-nowrap outline-none transition-colors focus-visible:outline-2 focus-visible:outline-focus",
        isHidden
          ? "border border-dashed border-fg-tertiary text-fg-tertiary hover:bg-fill"
          : "bg-fill text-fg-secondary hover:bg-fill-strong hover:text-fg",
      )}
      dir="ltr"
      onClick={onPress}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
      style={placed(scale, line.start, line.end)}
      type="button"
    >
      {isSpread ? (
        // Inside the pill's padding, each word holds the stretch from when
        // it is said to when the next is, so a word sits where it is said,
        // never runs into the next, and the last stops at the pill's end.
        <span aria-hidden className="absolute inset-y-0 inset-x-1.5">
          {line.words.map(({ at, word }, index) => {
            const next = line.words[index + 1]?.at ?? line.end;
            const share = (seconds: number) =>
              (scale.toX(seconds) - from) / length;
            return (
              <span
                className="absolute inset-y-0 flex items-center"
                dir="auto"
                key={at}
                style={{
                  insetInlineStart: `${String(share(at) * 100)}%`,
                  width: `${String((share(next) - share(at)) * 100)}%`,
                }}
              >
                <span className="truncate">{word}</span>
              </span>
            );
          })}
        </span>
      ) : px > TEXT_PX ? (
        <span aria-hidden className="truncate" dir="auto">
          {line.text}
        </span>
      ) : null}
    </button>
  );
}
