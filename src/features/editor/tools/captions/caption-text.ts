// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  TranscriptCorrection,
  TranscriptRange,
} from "../tool-panel-transcript";
import { transcriptInPlaybackOrder } from "../transcript/transcript-playback-order";
import {
  TranscriptUnit,
  transcriptUnits,
} from "../transcript/transcript-units";
import { firstIndex, TranscriptWord } from "../transcript/transcript-words";

import { breakIn } from "./caption-word-choices";

import type { Caption } from "../../../../bindings/Caption";
import type { CaptionBreak } from "../../../../bindings/CaptionBreak";

/** What the gap before a word holds: nothing, a break the captions make
 * there, or one placed by hand, to start a caption or to keep one going. */
export type CaptionGap = "break" | "join" | "none" | "split";

/** A word the timeline plays, as the Captions panel lays it out. */
export type CaptionTextWord = TranscriptUnit & {
  /** The caption showing it, or null for a word the captions leave out. */
  caption: number | null;
  /** What the caption shows of it, starred or without its punctuation as
   * the settings ask, in the case it was written; null where none does, or
   * where the captions were made before its latest correction. */
  captionText: string | null;
  gap: CaptionGap;
  /** Whether its caption sets it apart as its key word. */
  isKeyWord: boolean;
};

/** The transcript as the Captions panel lays it out: every word the
 * timeline plays, in its order, the paragraphs as runs of them, and where
 * in the source each caption is said. Until the captions are made, nothing
 * is marked, and every word shows plain. */
export type CaptionText = {
  captions: readonly TranscriptRange[];
  isMarked: boolean;
  /** Each paragraph, as `[start, end)` word indices. */
  paragraphs: readonly { end: number; start: number }[];
  words: readonly CaptionTextWord[];
};

type Placed = TranscriptRange & {
  caption: number;
  /** Whether the accent sets it apart. */
  emphasis: boolean;
  /** Whether a space parts it from the word after in its caption. */
  spaced: boolean;
  text: string;
};

/** The caption words heard over `range`, in order: one for a word, more for
 * a correction the captions set as several. Captions take a correction's
 * words as one, as the transcript does, so a word and its caption words
 * always overlap. */
const placedOver = (placed: readonly Placed[], range: TranscriptRange) => {
  const over: Placed[] = [];
  let at =
    firstIndex(placed.length, (i) => placed[i].startMs >= range.endMs) - 1;
  while (at >= 0 && placed[at].endMs > range.startMs)
    over.unshift(placed[at--]);
  return over;
};

/** The characters of `text`, by code point: a capital and its small letter
 * are one each, which is all the lining up below compares. */
const charactersOf = (text: string) => Array.from(text);

/**
 * `shown`, a caption's text, where it was made of `said`: it lines up with
 * `said` letter for letter, a star standing for one letter and punctuation
 * left out. Each letter takes the case `said` has it in, whatever case the
 * captions set it in: the captions change case only for All caps, and the
 * captions last made can still be in capitals for a moment after it is
 * turned off. Null where they do not line up: the captions were made before
 * a correction the transcript has since taken, and `said` is the truer text
 * until they are made again.
 */
function asSaid(shown: string, said: string) {
  const letters = charactersOf(said);
  let at = 0;
  let cased = "";
  for (const character of charactersOf(shown)) {
    while (
      at < letters.length &&
      (character === "*"
        ? !/\p{L}/u.test(letters[at])
        : letters[at].toUpperCase() !== character.toUpperCase())
    )
      at++;
    if (at === letters.length) return null;
    cased += character === "*" ? character : letters[at];
    at++;
  }
  return cased;
}

/**
 * The words the timeline plays, in the order it plays them and corrected,
 * each with the caption that shows it and what the gap before it holds.
 * Words the edit cuts are left out, as the captions leave them; words the
 * captions leave out, such as hidden fillers, stay. A break the captions
 * make is one between two shown words of different captions; one placed by
 * hand shows whatever the captions made of it.
 */
export function captionText(
  words: readonly TranscriptWord[],
  transcript: {
    corrections: readonly TranscriptCorrection[];
    durationMs: number;
    played: readonly TranscriptRange[];
    removed: readonly TranscriptRange[];
  },
  {
    breaks,
    captions,
    keyWords = false,
  }: {
    breaks: readonly CaptionBreak[];
    captions: readonly Caption[];
    /** Whether the captions set a key word of each apart; none do unless
     * asked. */
    keyWords?: boolean;
  },
): CaptionText {
  const placed = captions
    .flatMap((caption, index) =>
      caption.lines.flat().map((word) => ({
        caption: index,
        emphasis: word.emphasis,
        endMs: word.sourceEndMs,
        spaced: caption.spaced,
        startMs: word.sourceStartMs,
        text: word.text,
      })),
    )
    .sort((a, b) => a.startMs - b.startMs);
  const spans = captions.map((caption) => {
    const said = caption.lines.flat();
    return {
      endMs: Math.max(...said.map((word) => word.sourceEndMs)),
      startMs: Math.min(...said.map((word) => word.sourceStartMs)),
    };
  });
  const heard = transcriptUnits(words, transcript.corrections, {
    durationMs: transcript.durationMs,
    removed: transcript.removed,
  });
  const { units } = transcriptInPlaybackOrder(heard, transcript.played);

  const isMarked = captions.length > 0;
  const laid: CaptionTextWord[] = [];
  const paragraphs: { end: number; start: number }[] = [];
  let start = 0;
  // The caption of the last shown word, which a different one breaks from.
  let shownBefore: number | null = null;
  for (const unit of units) {
    if (unit.kind === "word" && !unit.removed) {
      const over = isMarked ? placedOver(placed, unit) : [];
      const caption = over.length > 0 ? over[over.length - 1].caption : null;
      const set =
        over.length > 0
          ? over.map((word) => word.text).join(over[0].spaced ? " " : "")
          : null;
      const before = laid[laid.length - 1] as CaptionTextWord | undefined;
      const placedBreak = before
        ? breakIn(breaks, {
            after: unit,
            before,
            durationMs: transcript.durationMs,
          })
        : null;
      laid.push({
        ...unit,
        caption,
        captionText: set === null ? null : asSaid(set, unit.text),
        gap: placedBreak
          ? placedBreak.kind
          : caption !== null && shownBefore !== null && shownBefore !== caption
            ? "break"
            : "none",
        isKeyWord: keyWords && over.some((word) => word.emphasis),
      });
      if (caption !== null) shownBefore = caption;
    }
    if (unit.endsParagraph && laid.length > start) {
      paragraphs.push({ end: laid.length, start });
      start = laid.length;
    }
  }
  if (laid.length > start) paragraphs.push({ end: laid.length, start });
  return { captions: spans, isMarked, paragraphs, words: laid };
}

/** The caption being said at `ms` in the source, or null between them. */
export const captionSaidAt = (
  captions: readonly TranscriptRange[],
  ms: number,
) => {
  const index = captions.findIndex(
    (span) => span.startMs <= ms && ms < span.endMs,
  );
  return index >= 0 ? index : null;
};

/**
 * Where pressing word `index` moves the playhead, in the source, from
 * `playheadMs`: the middle of its caption, where the caption is fully on
 * screen, or nowhere while the playhead is in that caption already. A word
 * in no caption seeks to its own start.
 */
export const captionSeekMs = (
  text: CaptionText,
  index: number,
  playheadMs: number,
): number | null => {
  const word = text.words[index];
  if (word.caption === null) return word.startMs;
  const caption = text.captions[word.caption];
  if (caption.startMs <= playheadMs && playheadMs < caption.endMs) return null;
  return (caption.startMs + caption.endMs) / 2;
};
