// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { boundsAnchor } from "../../../popup-panel/use-popup-menu";
import { CaptionsPatch } from "../../recording/captions/recording-captions";
import { normalizeText } from "../transcript/transcript-units";

import { CaptionText, CaptionTextWord } from "./caption-text";
import { CaptionTextPart, CaptionWordEdit } from "./caption-text-view";
import { canBreakIn, withBreak, withChoice } from "./caption-word-choices";
import {
  captionWordParts,
  hasLetters,
  withCaptionPunctuation,
} from "./caption-word-parts";
import {
  CaptionMenuAction,
  captionMenuItems,
  gapMenuItems,
  useCaptionMenu,
} from "./use-caption-menu";

import type { RecordingCaptions } from "../../../../bindings/RecordingCaptions";
import type { TranscriptMark } from "../transcript/use-transcript-marks";

/**
 * What can be done to a word of the caption text, wherever it is laid out:
 * the panel's caption editor and the Script island alike. Its spelling
 * corrected, in place or back to what was heard, which is the transcript's
 * and so `onCorrect`'s; and, as the captions' own edits, which undo with
 * them, a break placed or taken away in the gap before it, the word hidden
 * or shown, and made its caption's key word or kept plain. Pressed Return
 * edits it, Backspace hides or shows it, and a right click or the menu key
 * opens the app's own menu of the rest. What it returns is the text view's
 * part (`CaptionTextView`).
 */
export function useCaptionWordEditing({
  captions,
  durationMs,
  isDisabled,
  marks,
  onChange,
  onCorrect,
  text,
}: {
  captions: RecordingCaptions;
  /** The recording's length, which breaks and choices are shares of. */
  durationMs: number;
  isDisabled: boolean;
  marks: ReadonlyMap<number, TranscriptMark>;
  onChange: (patch: CaptionsPatch) => void;
  /** Puts `corrected` in for `word` in the transcript. */
  onCorrect: (word: CaptionTextWord, corrected: string) => void;
  /** Null until the transcript's words are known. */
  text: CaptionText | null;
}) {
  const [editing, setEditing] = useState<CaptionWordEdit | null>(null);
  const breaks = captions.breaks ?? [];
  const choices = captions.wordChoices ?? [];
  const emphasis = captions.emphasisChoices ?? [];
  const keyWords = captions.style.accent === "word";
  const { settings } = captions;

  /** Whether the rules alone leave `word` out of the captions. */
  const isRuledOut = (word: number) => {
    const mark = marks.get(word);
    return (
      (mark === "filler" && settings.hideFillers) ||
      (mark === "profanity" &&
        settings.censorProfanity &&
        settings.profanity === "remove")
    );
  };
  const isShown = (word: number) =>
    text !== null && text.words[word].caption !== null;
  const gapBefore = (word: number) => {
    if (!text || word === 0 || !isShown(word)) return null;
    const gap = {
      after: text.words[word],
      before: text.words[word - 1],
      durationMs,
    };
    return canBreakIn(gap) ? text.words[word].gap : null;
  };
  const act = (action: CaptionMenuAction, index: number) => {
    if (!text || isDisabled) return;
    if (action === "edit") {
      setEditing({ start: { point: null }, word: index });
      return;
    }
    const word = text.words[index];
    // Back to what was heard, which the transcript keeps as no correction.
    if (action === "restore-spelling") {
      onCorrect(word, word.heard);
      return;
    }
    if (action === "hide" || action === "show") {
      // A word stays left to the rules wherever they already do as asked,
      // so the edit keeps only the choices that change something.
      const ruledOut = isRuledOut(index);
      const kind =
        action === "hide"
          ? ruledOut
            ? null
            : "hidden"
          : ruledOut
            ? "shown"
            : null;
      onChange({
        wordChoices: withChoice(choices, { durationMs, word }, kind),
      });
      return;
    }
    if (action === "make-key-word" || action === "not-key-word") {
      // Kept plain by hand rather than left to the rule, which would pick
      // the word again were it its caption's longest.
      onChange({
        emphasisChoices: withChoice(
          emphasis,
          { durationMs, word },
          action === "make-key-word" ? "emphasized" : "plain",
        ),
      });
      return;
    }
    if (index === 0) return;
    const gap = { after: word, before: text.words[index - 1], durationMs };
    const kind = action === "clear-break" ? null : action;
    onChange({ breaks: withBreak(breaks, gap, kind) });
  };
  const openMenu = useCaptionMenu(act);

  return {
    editing,
    /** Whether anything was placed here by hand, which Reset takes away;
     * spelling is the transcript's, so it stays. */
    hasPlaced: breaks.length > 0 || choices.length > 0 || emphasis.length > 0,
    /** `typed` is what the edit left in place of the word, without the
     * punctuation either side (`captionWordParts`), which it keeps. */
    onEditEnd: (index: number, typed: string | null) => {
      setEditing(null);
      if (typed === null || !text) return;
      const word = text.words[index];
      const left = normalizeText(typed);
      // No letters left is the word taken out of the captions, as Backspace
      // takes it; the recording keeps it. A word that was only a mark goes
      // only when emptied.
      if (left === "" || (hasLetters(word.text) && !hasLetters(left))) {
        if (text.isMarked && isShown(index)) act("hide", index);
        return;
      }
      const corrected = withCaptionPunctuation(
        captionWordParts(word.text),
        left,
      );
      if (corrected !== normalizeText(word.text)) onCorrect(word, corrected);
    },
    onEditStart: setEditing,
    onHideToggle: (word: number) => {
      act(isShown(word) ? "hide" : "show", word);
    },
    onMenu: (word: number, bounds: DOMRect, part: CaptionTextPart) => {
      if (!text) return;
      const gap = gapBefore(word);
      // A break or the space before a word is asked only about that gap.
      const items =
        part === "gap"
          ? gapMenuItems(gap)
          : captionMenuItems({
              gap,
              isCorrected: text.words[word].corrected,
              isKeyWord:
                keyWords && isShown(word) ? text.words[word].isKeyWord : null,
              isShown: isShown(word),
            });
      if (items.length > 0) void openMenu(word, boundsAnchor(bounds), items);
    },
    /** Takes away every break, choice and key word placed by hand. */
    reset: () => {
      onChange({ breaks: [], emphasisChoices: [], wordChoices: [] });
    },
  };
}
