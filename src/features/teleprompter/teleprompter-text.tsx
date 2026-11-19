// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { motion, useReducedMotion, useSpring } from "motion/react";
import { useLayoutEffect, useRef, useState } from "react";
import { TextArea, TextField } from "react-aria-components";

import { t } from "../../i18n/i18n";
import { cn } from "../../lib/styling";
import {
  type PopupMenuAnchor,
  pointerAnchor,
} from "../popup-panel/use-popup-menu";

import { columnStyle } from "./column-style";
import { wordStartAt } from "./cues/cue-anchors";
import { CueMark } from "./cues/cue-mark";
import { READING_LINE, readingLineMiddle } from "./reading-line";
import { ScriptText } from "./script-words";
import { TeleprompterColumn } from "./teleprompter-column";

/** How long after the last turn of the wheel the place settles on the line
 * scrolled to, in milliseconds. */
const SCROLL_SETTLE_MS = 160;

/**
 * The script, large, moved up so the word being read (`place`, the next
 * word's index, fractional between words) stands on the reading line,
 * easing there rather than jumping, as the voice moves on in steps. What
 * has been read is dimmed. The script is edited where it is read: a field
 * holding the same text lies over it letter for letter, its own text
 * unseen, so the caret and the selection fall on the words shown. Scrolled
 * by hand, the text follows the wheel, and the place settles on the line
 * left on the reading line. The text runs in a column whose edges are
 * dragged to set its width (`TeleprompterColumn`). A cue is marked at the
 * column's end beside its word, which opens the word's menu, as a right
 * click on the word does.
 */
export function TeleprompterText({
  columnWidth,
  cues,
  isPlaying,
  onColumnWidthChange,
  onEdit,
  onScrollTo,
  onWordMenu,
  place,
  script,
  text,
  textSize,
}: {
  /** The widest the text runs, in points; null fills the frame. */
  columnWidth: number | null;
  /** The cues, marked beside their words. */
  cues: readonly { at: number; title: string }[];
  isPlaying: boolean;
  onColumnWidthChange: (width: number | null) => void;
  onEdit: (text: string) => void;
  /** The place scrolled to by hand, as a word's index. */
  onScrollTo: (index: number) => void;
  /** Opens the menu of the word starting at `at` in the text. */
  onWordMenu: (anchor: PopupMenuAnchor, at: number) => void;
  place: number;
  script: ScriptText;
  /** The script as written. */
  text: string;
  /** In points. */
  textSize: number;
}) {
  const frameRef = useRef<HTMLDivElement>(null);
  const columnRef = useRef<HTMLDivElement>(null);
  const wordsRef = useRef(new Map<number, HTMLSpanElement>());
  const isReduced = useReducedMotion() ?? false;
  const offset = useSpring(0, { bounce: 0, visualDuration: 0.35 });
  // Bumped when the frame is resized, which moves the text at once, and
  // when a scroll by hand settles, which eases it onto the line.
  const [resized, setResized] = useState(0);
  const [settled, setSettled] = useState(0);
  const jumpsRef = useRef(true);
  const scrollingRef = useRef<number | undefined>(undefined);
  const read = Math.floor(place);

  // The frame resized, or the column in it, rewraps the text: it is put
  // back on the reading line at once.
  useLayoutEffect(() => {
    const frame = frameRef.current;
    const column = columnRef.current;
    if (!frame || !column) return;
    const observer = new ResizeObserver(() => {
      jumpsRef.current = true;
      setResized((count) => count + 1);
    });
    observer.observe(frame);
    observer.observe(column);
    return () => {
      observer.disconnect();
    };
  }, []);

  const readingY = () => (frameRef.current?.clientHeight ?? 0) * READING_LINE;
  const topOf = (index: number) => wordsRef.current.get(index)?.offsetTop;

  useLayoutEffect(() => {
    if (scrollingRef.current !== undefined) return;
    const last = script.spoken.length - 1;
    const at = topOf(Math.min(read, last)) ?? 0;
    // Between two words on different lines the text moves on in proportion,
    // so a steady pace scrolls smoothly rather than a line at a time.
    const next = topOf(Math.min(read + 1, last)) ?? at;
    const target = readingY() - (at + (next - at) * (place - read));
    if (isReduced || jumpsRef.current) offset.jump(target);
    else offset.set(target);
    jumpsRef.current = false;
  }, [isReduced, offset, place, read, resized, script, settled, textSize]);

  /** The first word on the line nearest the reading line, with the text
   * moved to `y`. */
  const wordAt = (y: number) => {
    const line = readingY() - y;
    let best = 0;
    let distance = Infinity;
    for (const [index, element] of wordsRef.current) {
      const away = Math.abs(element.offsetTop - line);
      if (away < distance || (away === distance && index < best)) {
        best = index;
        distance = away;
      }
    }
    return best;
  };

  /** Where the word under the point starts in the text, if one is: the
   * words are drawn under the field, which hit testing passes through. */
  const startOfWordAt = (x: number, y: number) => {
    const under = document.elementsFromPoint(x, y);
    for (const [index, element] of wordsRef.current)
      if (under.includes(element)) return script.starts[index];
    return undefined;
  };

  const marked = new Map(cues.map((cue) => [cue.at, cue.title]));
  const hasMarks = marked.size > 0;

  let lastWord = -1;
  // Each piece is keyed by where it starts in the text.
  let at = 0;
  return (
    // Its own compositing layer, so WebKit repaints under the fade at the
    // edges when the text inside it rewraps; without one, the text as it
    // was laid out last stays on screen beside the new.
    <div
      className="relative min-h-0 grow overflow-hidden mask-t-from-85% mask-b-from-70% will-change-transform"
      // A caret placed out of sight would have the browser scroll the
      // frame to it; the text is moved by its offset alone.
      onScroll={(event) => {
        event.currentTarget.scrollTop = 0;
      }}
      onWheel={(event) => {
        const lastTop = topOf(script.spoken.length - 1) ?? 0;
        const y = Math.min(
          readingY(),
          Math.max(readingY() - lastTop, offset.get() - event.deltaY),
        );
        offset.jump(y);
        window.clearTimeout(scrollingRef.current);
        scrollingRef.current = window.setTimeout(() => {
          scrollingRef.current = undefined;
          onScrollTo(wordAt(y));
          setSettled((count) => count + 1);
        }, SCROLL_SETTLE_MS);
      }}
      ref={frameRef}
    >
      <TeleprompterColumn
        frameRef={frameRef}
        isPlaying={isPlaying}
        onWidthChange={onColumnWidthChange}
        readingTop={readingLineMiddle(textSize)}
        width={columnWidth}
      />
      <motion.div
        className="relative mx-auto font-semibold leading-snug"
        ref={columnRef}
        style={{ ...columnStyle(columnWidth), fontSize: textSize, y: offset }}
      >
        <div
          aria-hidden
          // An empty script leaves room for the placeholder's lines.
          className={cn(
            "px-inset break-words whitespace-pre-wrap",
            text === "" ? "min-h-[3lh]" : "min-h-lh",
            hasMarks && "pe-8",
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
                  "transition-colors duration-200",
                  lastWord >= 0 && lastWord < read
                    ? "text-fg-secondary/50"
                    : "text-fg",
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
                {piece.kind === "word" && marked.has(key) ? (
                  <CueMark
                    onPress={(anchor) => {
                      onWordMenu(anchor, key);
                    }}
                    title={marked.get(key) ?? ""}
                  />
                ) : null}
                {piece.text}
              </span>
            );
          })}
          {/* A field shows a last empty line after a final line break,
              which text alone does not. */}
          {text.endsWith("\n") ? "\u200b" : null}
        </div>
        <TextField
          aria-label={t("teleprompter-script-label")}
          className="absolute inset-0"
          onChange={onEdit}
          value={text}
        >
          {/* The browser's own text rendering for a field is not the
              page's, which would set the words a touch apart from those
              under it along a line. */}
          <TextArea
            className={cn(
              "size-full resize-none overflow-hidden bg-transparent px-inset break-words whitespace-pre-wrap text-transparent caret-fg outline-none [text-rendering:inherit] placeholder:text-fg-tertiary",
              hasMarks && "pe-8",
            )}
            dir="auto"
            onContextMenu={(event) => {
              event.preventDefault();
              onWordMenu(
                pointerAnchor(event.clientX, event.clientY),
                startOfWordAt(event.clientX, event.clientY) ??
                  wordStartAt(script, event.currentTarget.selectionStart),
              );
            }}
            placeholder={t("teleprompter-script-placeholder")}
          />
        </TextField>
      </motion.div>
    </div>
  );
}
