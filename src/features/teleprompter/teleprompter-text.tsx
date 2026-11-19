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
import { CueMarks, type MarkedCue } from "./cues/cue-marks";
import { type CuePlace, movedTo } from "./cues/place-cue";
import { PlacingPicture } from "./cues/placing-picture";
import { useCueDrag } from "./cues/use-cue-drag";
import { READING_LINE, readingLineMiddle } from "./reading-line";
import { ScriptLayer } from "./script-layer";
import { ScriptText } from "./script-words";
import { TeleprompterColumn } from "./teleprompter-column";
import { type ScreenPoint, startOfWordAt, wordStartNear } from "./word-hits";

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
 * column's end beside its word: clicked, it opens the word's menu, as a
 * right click on the word does, and dragged it moves to another word. A cue
 * made in an editor waits for its word, its picture following the pointer
 * over the text, until a click picks one.
 */
export function TeleprompterText({
  columnWidth,
  cues,
  isPlaying,
  onColumnWidthChange,
  onCuesMove,
  onEdit,
  onPlace,
  onScrollTo,
  onWordMenu,
  place,
  placing,
  script,
  text,
  textSize,
}: {
  /** The widest the text runs, in points; null fills the frame. */
  columnWidth: number | null;
  /** The cues, marked beside their words. */
  cues: readonly MarkedCue[];
  isPlaying: boolean;
  onColumnWidthChange: (width: number | null) => void;
  /** A cue dragged to another word, at that word. */
  onCuesMove: (moved: readonly CuePlace[]) => void;
  onEdit: (text: string) => void;
  /** The word starting `at` picked for the cue being placed. */
  onPlace: (at: number) => void;
  /** The place scrolled to by hand, as a word's index. */
  onScrollTo: (index: number) => void;
  /** Opens the menu of the word starting at `at` in the text. */
  onWordMenu: (anchor: PopupMenuAnchor, at: number) => void;
  place: number;
  /** A cue waiting for its word, with its picture once drawn. */
  placing: { picture: ImageBitmap | string | null } | null;
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

  /** The word a point is over, or the first of the line beside it. */
  const landingAt = (point: ScreenPoint) =>
    wordStartNear(wordsRef.current, script, point);
  // A cue dragged to another word is let go there; the others keep their
  // words, and their marks make room for its mark (`CueMarks`). A word
  // holds one cue, so a word holding another is passed over, the drag
  // staying on the last word it could land on.
  const { begin, drag } = useCueDrag({
    landingAt: (point, id) => {
      const at = landingAt(point);
      return cues.some((cue) => cue.at === at && cue.id !== id)
        ? undefined
        : at;
    },
    onMove: (id, at) => {
      if (cues.some((cue) => cue.id === id && cue.at !== at))
        onCuesMove([{ at, id }]);
    },
  });
  const marks = drag ? movedTo(cues, [drag]) : cues;
  const hasMarks = cues.length > 0;
  // Where the pointer is over the text while a cue is being placed, and the
  // word it would place it on.
  const [pointing, setPointing] = useState<{
    at: number | null;
    x: number;
    y: number;
  } | null>(null);

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
        <ScriptLayer
          hasMarks={hasMarks}
          pointedAt={drag?.at ?? (placing ? (pointing?.at ?? null) : null)}
          read={read}
          script={script}
          text={text}
          wordsRef={wordsRef}
        />
        <CueMarks
          dragged={drag}
          marks={marks}
          onMarkPress={(cue, press, anchor) => {
            begin(cue, press, () => {
              onWordMenu(anchor, cue.at);
            });
          }}
          script={script}
          wordsRef={wordsRef}
        />
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
              hasMarks && "pe-12",
              placing && "cursor-pointer",
            )}
            dir="auto"
            onContextMenu={(event) => {
              event.preventDefault();
              onWordMenu(
                pointerAnchor(event.clientX, event.clientY),
                startOfWordAt(wordsRef.current, script, event) ??
                  wordStartAt(script, event.currentTarget.selectionStart),
              );
            }}
            onPointerDown={(event) => {
              // Placing a cue, a press picks its word rather than the caret.
              if (!placing || event.button !== 0) return;
              event.preventDefault();
              const to = landingAt(event);
              if (to !== undefined) onPlace(to);
            }}
            onPointerLeave={() => {
              setPointing(null);
            }}
            onPointerMove={(event) => {
              if (!placing) return;
              const frame = frameRef.current?.getBoundingClientRect();
              setPointing({
                at: landingAt(event) ?? null,
                x: event.clientX - (frame?.left ?? 0),
                y: event.clientY - (frame?.top ?? 0),
              });
            }}
            placeholder={t("teleprompter-script-placeholder")}
          />
        </TextField>
      </motion.div>
      {placing && pointing ? (
        <PlacingPicture
          picture={placing.picture}
          x={pointing.x}
          y={pointing.y}
        />
      ) : null}
    </div>
  );
}
