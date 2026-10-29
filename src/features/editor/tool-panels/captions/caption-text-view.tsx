// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Fragment,
  KeyboardEvent,
  MouseEvent,
  useId,
  useRef,
  useState,
} from "react";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";
import { EditStart } from "../transcript/transcript-edit-span";
import { usePlayingWord } from "../transcript/use-playing-word";

import {
  captionSaidAt,
  captionSeekMs,
  CaptionText,
  CaptionTextWord,
} from "./caption-text";
import { beforeCaptionWord, captionWordClass } from "./caption-text-words";
import { CaptionWordEditSpan } from "./caption-word-edit-span";
import { CaptionPaint, useCaptionPaint } from "./use-caption-paint";

import type { TranscriptMark } from "../transcript/use-transcript-marks";

/** A word being edited in place, and how its edit began. */
export type CaptionWordEdit = { start: EditStart; word: number };

/** What a right click landed on: a word, or the break or space before it. */
export type CaptionTextPart = "gap" | "word";

/** The parts of a caption's line within `root`: its words and the spaces
 * between them, in every paragraph it reaches. */
const partsOf = (root: HTMLElement | null, caption: number) => [
  ...(root?.querySelectorAll<HTMLElement>(
    `[data-caption="${String(caption)}"]`,
  ) ?? []),
];

/**
 * The transcript as the captions break it: its paragraphs, a mark wherever
 * one caption ends and the next begins and in another colour where that was
 * placed by hand, the caption being said tinted and kept in view, and the
 * one under the pointer filled. A press seeks to the word's caption, as
 * `captionSeekMs` says, and a double-click edits the word in place;
 * Command-click hides or shows a word, and Command-drag every word it
 * reaches (`useCaptionPaint`). The arrows move between words, Return
 * edits, Backspace hides or shows a word, and the menu key or a right
 * click opens what else it can do; a right click on a break or a space
 * asks only about that gap.
 */
export function CaptionTextView({
  className,
  editing,
  isDisabled,
  marks,
  onEditEnd,
  onEditStart,
  onHideToggle,
  onMenu,
  onPaint,
  onSeek,
  playheadMs,
  rootClassName,
  text,
}: {
  /** The word being edited in place and how its edit began, if one is. */
  editing: CaptionWordEdit | null;
  isDisabled: boolean;
  marks: ReadonlyMap<number, TranscriptMark>;
  /** The text the word was left holding, or null to put it back. */
  onEditEnd: (word: number, typed: string | null) => void;
  onEditStart: (edit: CaptionWordEdit) => void;
  onHideToggle: (word: number) => void;
  /** `word` is the word clicked, or the one after the gap clicked. */
  onMenu: (word: number, bounds: DOMRect, part: CaptionTextPart) => void;
  onPaint: (paint: CaptionPaint) => void;
  onSeek: (ms: number) => void;
  playheadMs: number;
  text: CaptionText;
  /** The text's inset within the box, over the panel's own. */
  className?: string;
  /** The scrolling box's size and fill, over the panel's own: a box of
   * the panel's fill, scrolling past 16 rem. */
  rootClassName?: string;
}) {
  const id = useId();
  const rootRef = useRef<HTMLDivElement>(null);
  const hoveredRef = useRef<number | null>(null);
  const [active, setActive] = useState(0);
  const { onPointerDown, painted } = useCaptionPaint({
    isEnabled: !isDisabled && !editing && text.isMarked,
    isHidden: (word) => text.words[word].caption === null,
    onPaint,
  });
  /** `word` as a Command-drag under way will leave it. */
  const shown = (word: CaptionTextWord, index: number) =>
    painted && painted.first <= index && index <= painted.last
      ? { ...word, caption: painted.hide ? null : (word.caption ?? -1) }
      : word;
  const focused = Math.min(active, text.words.length - 1);
  const playing = text.isMarked
    ? captionSaidAt(text.captions, playheadMs)
    : null;
  const markScrolled = usePlayingWord(
    (caption) => partsOf(rootRef.current, caption),
    { content: text, current: playing ?? -1, isChoosing: false },
  );
  const wordId = (index: number) => `${id}-word-${index.toString()}`;
  const hover = (caption: number | null) => {
    if (hoveredRef.current === caption) return;
    if (hoveredRef.current !== null)
      for (const part of partsOf(rootRef.current, hoveredRef.current))
        part.toggleAttribute("data-hovered", false);
    hoveredRef.current = caption;
    if (caption !== null)
      for (const part of partsOf(rootRef.current, caption))
        part.toggleAttribute("data-hovered", true);
  };
  /** The word, or the gap before it (`data-gap`), the event is over. */
  const indexAt = (event: MouseEvent, part: CaptionTextPart) => {
    const index = (event.target as HTMLElement).closest<HTMLElement>(
      `[data-${part}]`,
    )?.dataset[part];
    return index === undefined ? null : Number(index);
  };
  const wordAt = (event: MouseEvent) => indexAt(event, "word");
  const boundsOf = (index: number) =>
    document.getElementById(wordId(index))?.getBoundingClientRect() ?? null;
  const moveTo = (index: number) => {
    const next = Math.min(Math.max(index, 0), text.words.length - 1);
    setActive(next);
    document.getElementById(wordId(next))?.scrollIntoView({ block: "nearest" });
  };
  const onKeyDown = (event: KeyboardEvent) => {
    // The word being edited has the keys to itself.
    if (editing) return;
    const isRtl =
      rootRef.current !== null &&
      getComputedStyle(rootRef.current).direction === "rtl";
    const step = { ArrowLeft: isRtl ? 1 : -1, ArrowRight: isRtl ? -1 : 1 }[
      event.key
    ];
    if (step !== undefined) moveTo(focused + step);
    else if (event.key === "Home") moveTo(0);
    else if (event.key === "End") moveTo(text.words.length - 1);
    else if (!isDisabled && event.key === "Enter")
      onEditStart({ start: { point: null }, word: focused });
    else if (
      !isDisabled &&
      text.isMarked &&
      (event.key === "Backspace" || event.key === "Delete")
    )
      onHideToggle(focused);
    else if (
      !isDisabled &&
      text.isMarked &&
      (event.key === "ContextMenu" || (event.key === "F10" && event.shiftKey))
    ) {
      const bounds = boundsOf(focused);
      if (bounds) onMenu(focused, bounds, "word");
    } else return;
    event.preventDefault();
  };

  return (
    <ScrollArea
      constrainHeight
      rootClassName={cn("max-h-64 rounded-control bg-fill", rootClassName)}
    >
      <div
        aria-activedescendant={
          text.words.length > 0 ? wordId(focused) : undefined
        }
        aria-label={t("editor-panels-captions-text")}
        className={cn(
          "group flex cursor-default flex-col gap-separate p-inset-sm text-body text-fg outline-none select-none",
          className,
        )}
        data-owns-text-keys
        dir="auto"
        onClick={(event) => {
          const word = wordAt(event);
          // A Command-click hides or shows the word instead.
          if (word === null || editing || event.metaKey || event.ctrlKey)
            return;
          setActive(word);
          const seek = captionSeekMs(text, word, playheadMs);
          if (seek !== null) onSeek(seek);
        }}
        onContextMenu={(event) => {
          event.preventDefault();
          if (isDisabled || editing) return;
          const word = wordAt(event);
          const gap = word === null ? indexAt(event, "gap") : null;
          const index = word ?? gap;
          if (index === null) return;
          setActive(index);
          onMenu(
            index,
            new DOMRect(event.clientX, event.clientY, 1, 1),
            word === null ? "gap" : "word",
          );
        }}
        onDoubleClick={(event) => {
          const word = wordAt(event);
          if (word === null || isDisabled || editing) return;
          setActive(word);
          onEditStart({
            start: { point: { x: event.clientX, y: event.clientY } },
            word,
          });
        }}
        onKeyDown={onKeyDown}
        onPointerDown={onPointerDown}
        onPointerLeave={() => {
          hover(null);
        }}
        onPointerOver={(event) => {
          const caption = (event.target as HTMLElement).closest<HTMLElement>(
            "[data-caption]",
          )?.dataset.caption;
          hover(caption === undefined ? null : Number(caption));
        }}
        onWheel={markScrolled}
        ref={rootRef}
        role="listbox"
        tabIndex={0}
      >
        {text.paragraphs.map(({ end, start }) => (
          <p key={start}>
            {text.words.slice(start, end).map((word, offset) => {
              const index = start + offset;
              return (
                <Fragment key={index}>
                  {beforeCaptionWord(
                    word,
                    index,
                    offset === 0 ? null : text.words[index - 1],
                  )}
                  {editing?.word === index ? (
                    // The word is edited without its punctuation, which
                    // stays either side of it (`captionWordParts`).
                    <CaptionWordEditSpan
                      id={wordId(index)}
                      index={index}
                      onFinish={(typed) => {
                        onEditEnd(index, typed);
                        // Focus comes back to the text only where the edit
                        // left it nowhere, as Enter and Escape do. A press
                        // that ended it has already put focus where it
                        // pressed: taking it back by script would carry the
                        // edit's focus ring over, which a press never shows.
                        requestAnimationFrame(() => {
                          const active = document.activeElement;
                          if (active === null || active === document.body)
                            rootRef.current?.focus({ preventScroll: true });
                        });
                      }}
                      start={editing.start}
                      text={word.text}
                    />
                  ) : (
                    <span
                      aria-selected={index === focused}
                      className={captionWordClass(
                        shown(word, index),
                        text.isMarked,
                        marks.get(index),
                      )}
                      data-caption={word.caption ?? undefined}
                      data-word={index}
                      id={wordId(index)}
                      role="option"
                    >
                      {word.captionText ?? word.text}
                    </span>
                  )}
                </Fragment>
              );
            })}
          </p>
        ))}
      </div>
    </ScrollArea>
  );
}
