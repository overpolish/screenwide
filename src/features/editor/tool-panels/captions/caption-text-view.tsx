// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Fragment,
  KeyboardEvent,
  MouseEvent,
  ReactNode,
  useId,
  useRef,
  useState,
} from "react";

import { ScrollArea } from "../../../../components/base/scroll-area/scroll-area";
import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import {
  EditStart,
  TranscriptEditSpan,
} from "../transcript/transcript-edit-span";
import { usePlayingWord } from "../transcript/use-playing-word";

import { captionSaidAt, CaptionText, CaptionTextWord } from "./caption-text";

import type { TranscriptMark } from "../transcript/use-transcript-marks";

/** A word being edited in place, and how its edit began. */
export type CaptionWordEdit = { start: EditStart; word: number };

/** The parts of a caption's line within `root`: its words and the spaces
 * between them, in every paragraph it reaches. */
const partsOf = (root: HTMLElement | null, caption: number) => [
  ...(root?.querySelectorAll<HTMLElement>(
    `[data-caption="${String(caption)}"]`,
  ) ?? []),
];

// A caption's words and the spaces between take a quiet fill while the
// pointer is over any of them, and the accent's tint while it is said.
// Words the captions leave out read faded and struck through, as cut words
// do in the transcript; fillers and profanity they keep take the warning and
// error colours the Transcript panel marks them in, and words corrected by
// hand its quiet underline. The word the keys are on takes the focus ring
// while the text has focus.
const CAPTION_PART =
  "data-hovered:bg-fill-secondary data-current:bg-primary-tint";
const wordClass = (
  word: CaptionTextWord,
  isMarked: boolean,
  mark: TranscriptMark | undefined,
) =>
  cn(
    "group-focus-visible:aria-selected:ring-2 group-focus-visible:aria-selected:ring-focus-ring",
    isMarked && word.caption === null
      ? "text-content-fg-tertiary line-through"
      : cn(
          CAPTION_PART,
          mark === "filler" && "text-warning",
          mark === "profanity" && "text-error",
          word.corrected &&
            "underline decoration-content-fg-tertiary underline-offset-4",
        ),
  );

/** What sits before a word: nothing for a paragraph's first, the mark of a
 * break, or a space, carried in its caption's colour where it runs on. */
const before = (
  word: CaptionTextWord,
  isFirst: boolean,
  prior: CaptionTextWord | undefined,
): ReactNode => {
  if (word.gap === "split" || word.gap === "join")
    return (
      // A no-break space ties the mark to the words before it, so a wrapped
      // line ends on it rather than opening with it.
      <span aria-hidden className="text-accent-heading">
        {word.gap === "split" ? "\u00a0| " : "\u00a0\u203f "}
      </span>
    );
  if (isFirst) return null;
  if (word.gap === "break")
    return (
      <span aria-hidden className="text-content-fg-tertiary">
        {"\u00a0| "}
      </span>
    );
  return word.caption !== null && word.caption === prior?.caption ? (
    <span className={CAPTION_PART} data-caption={word.caption}>
      {" "}
    </span>
  ) : (
    " "
  );
};

/**
 * The transcript as the captions break it: its paragraphs, a mark wherever
 * one caption ends and the next begins and in another colour where that was
 * placed by hand, the caption being said tinted and kept in view, and the
 * one under the pointer filled. A press seeks to the start of the word's
 * caption, and a double-click edits the word in place. The arrows move
 * between words, Return edits, Backspace hides or shows a word, and the
 * menu key or a right click opens what else it can do.
 */
export function CaptionTextView({
  editing,
  isDisabled,
  marks,
  onEditEnd,
  onEditStart,
  onHideToggle,
  onMenu,
  onSeek,
  playheadMs,
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
  onMenu: (word: number, bounds: DOMRect) => void;
  onSeek: (ms: number) => void;
  playheadMs: number;
  text: CaptionText;
}) {
  const id = useId();
  const rootRef = useRef<HTMLDivElement>(null);
  const hoveredRef = useRef<number | null>(null);
  const [active, setActive] = useState(0);
  const focused = Math.min(active, text.words.length - 1);
  const playing = text.isMarked
    ? captionSaidAt(text.captions, playheadMs)
    : null;
  const markScrolled = usePlayingWord(
    (caption) => partsOf(rootRef.current, caption),
    { content: text, current: playing ?? -1, isChoosing: false },
  );
  const wordId = (index: number) => `${id}-word-${index.toString()}`;
  const seekTo = (index: number) => {
    const word = text.words[index];
    onSeek(
      word.caption === null
        ? word.startMs
        : text.captions[word.caption].startMs,
    );
  };
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
  const wordAt = (event: MouseEvent) => {
    const word = (event.target as HTMLElement).closest<HTMLElement>(
      "[data-word]",
    )?.dataset.word;
    return word === undefined ? null : Number(word);
  };
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
      if (bounds) onMenu(focused, bounds);
    } else return;
    event.preventDefault();
  };

  return (
    <ScrollArea
      constrainHeight
      rootClassName="max-h-64 rounded-control bg-fill-tertiary"
    >
      <div
        aria-activedescendant={
          text.words.length > 0 ? wordId(focused) : undefined
        }
        aria-label={t("editor-panels-captions-text")}
        className="group flex cursor-default flex-col gap-section p-control-inset text-body text-content-fg outline-none select-none"
        data-owns-text-keys
        dir="auto"
        onClick={(event) => {
          const word = wordAt(event);
          if (word === null || editing) return;
          setActive(word);
          seekTo(word);
        }}
        onContextMenu={(event) => {
          event.preventDefault();
          const word = wordAt(event);
          if (word === null || isDisabled || editing) return;
          setActive(word);
          onMenu(word, new DOMRect(event.clientX, event.clientY, 1, 1));
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
                  {before(word, offset === 0, text.words[index - 1])}
                  {editing?.word === index ? (
                    <TranscriptEditSpan
                      className="outline-none select-text"
                      id={wordId(index)}
                      index={index}
                      onFinish={(typed) => {
                        rootRef.current?.focus({ preventScroll: true });
                        onEditEnd(index, typed);
                      }}
                      start={editing.start}
                      text={word.text}
                    />
                  ) : (
                    <span
                      aria-selected={index === focused}
                      className={wordClass(
                        word,
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
