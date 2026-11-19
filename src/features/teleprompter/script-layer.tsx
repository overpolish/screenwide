// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { type RefObject } from "react";

import { cn } from "../../lib/styling";

import { ScriptText } from "./script-words";

// The script's words as they are read, under the field they are written in:
// what has been read dimmed, and a word being pointed at to drop or place a
// cue on picked out. Each piece is keyed by where it starts in the text.
export function ScriptLayer({
  hasMarks,
  pointedAt,
  read,
  script,
  text,
  wordsRef,
}: {
  /** Room is kept at the column's end for cues' marks. */
  hasMarks: boolean;
  /** Where the word a cue would go starts, if one is being pointed at. */
  pointedAt: number | null;
  /** The index of the next word to read. */
  read: number;
  script: ScriptText;
  text: string;
  /** Each word's element, by its index, kept as they are drawn. */
  wordsRef: RefObject<Map<number, HTMLSpanElement>>;
}) {
  let lastWord = -1;
  let at = 0;
  return (
    <div
      aria-hidden
      // An empty script leaves room for the placeholder's lines.
      className={cn(
        "px-inset break-words whitespace-pre-wrap",
        text === "" ? "min-h-[3lh]" : "min-h-lh",
        hasMarks && "pe-12",
      )}
      dir="auto"
    >
      {script.pieces.map((piece) => {
        if (piece.kind === "word") lastWord = piece.index;
        const key = at;
        at += piece.text.length;
        return (
          <span
            className={cn(
              "rounded-sm transition-colors duration-200",
              lastWord >= 0 && lastWord < read
                ? "text-fg-secondary/50"
                : "text-fg",
              piece.kind === "word" && key === pointedAt && "bg-fill-strong",
            )}
            key={key}
            ref={
              piece.kind === "word"
                ? (element) => {
                    if (element) wordsRef.current.set(piece.index, element);
                    else wordsRef.current.delete(piece.index);
                  }
                : undefined
            }
          >
            {piece.text}
          </span>
        );
      })}
      {/* A field shows a last empty line after a final line break, which
          text alone does not. */}
      {text.endsWith("\n") ? "\u200b" : null}
    </div>
  );
}
