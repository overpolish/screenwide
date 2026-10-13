// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo, useState } from "react";

import { Button } from "../../../../components/base/button/button";
import { t } from "../../../../i18n/i18n";
import { boundsAnchor } from "../../../popup-panel/use-popup-menu";
import { CaptionsPatch } from "../../recording/captions/recording-captions";
import { ToolPanelTranscript } from "../tool-panel-transcript";
import { useTranscriptChannel } from "../transcript/transcript-channel";
import { normalizeText } from "../transcript/transcript-units";
import { TranscriptWord } from "../transcript/transcript-words";
import { usePendingCorrections } from "../transcript/use-pending-corrections";
import {
  useTranscriptPlayheadFeed,
  useTranscriptWordsFeed,
} from "../transcript/use-transcript-feed";
import { useTranscriptMarks } from "../transcript/use-transcript-marks";

import { captionText, CaptionTextWord } from "./caption-text";
import { CaptionTextView, CaptionWordEdit } from "./caption-text-view";
import { canBreakIn, withBreak, withChoice } from "./caption-word-choices";
import { useCaptionList } from "./use-caption-list";
import {
  CaptionMenuAction,
  captionMenuItems,
  useCaptionMenu,
} from "./use-caption-menu";

import type { RecordingCaptions } from "../../../../bindings/RecordingCaptions";
import type { TranscriptCorrection } from "../tool-panel-transcript";

const NO_WORDS: readonly CaptionTextWord[] = [];
const NOTHING_HEARD: readonly TranscriptWord[] = [];

/**
 * The caption editor in the panel window: the transcript's words and the
 * playhead read from the editor, laid out by the captions last made, with
 * the fillers and profanity the Transcript panel marks. Breaks and hidden
 * words placed here are the captions' own and undo with the edit; a word
 * edited here is corrected in the transcript, as the Transcript panel
 * corrects it, so the two never disagree.
 */
export function CaptionEditor({
  captions,
  isDisabled,
  onChange,
  onCorrect,
  onSeek,
  transcript,
}: {
  captions: RecordingCaptions;
  isDisabled: boolean;
  onChange: (patch: CaptionsPatch) => void;
  onCorrect: (correction: TranscriptCorrection) => void;
  onSeek: (ms: number) => void;
  transcript: ToolPanelTranscript;
}) {
  useTranscriptPlayheadFeed();
  useTranscriptWordsFeed(transcript);
  const words = useTranscriptChannel((state) => state.words);
  const language = useTranscriptChannel((state) => state.language);
  const playheadMs = useTranscriptChannel((state) => state.playheadMs);
  const made = useCaptionList(transcript.artifactId);
  const [editing, setEditing] = useState<CaptionWordEdit | null>(null);
  // A correction shows as soon as it is typed, before the editor answers.
  const pending = usePendingCorrections(
    words ?? NOTHING_HEARD,
    transcript.corrections,
  );
  const corrections = pending.corrections;
  const { durationMs, played, removed } = transcript;
  const breaks = captions.breaks ?? [];
  const choices = captions.wordChoices ?? [];
  const allCaps = captions.style.allCaps;
  const text = useMemo(
    () =>
      words
        ? captionText(
            words,
            { corrections, durationMs, played, removed },
            { allCaps, breaks: captions.breaks ?? [], captions: made },
          )
        : null,
    [
      allCaps,
      captions.breaks,
      corrections,
      durationMs,
      made,
      played,
      removed,
      words,
    ],
  );
  const { marks } = useTranscriptMarks(
    text?.words ?? NO_WORDS,
    language,
    transcript.allowedWords,
  );
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
  /** Puts `corrected` in for `word` in the transcript, shown here at once. */
  const correct = (word: CaptionTextWord, corrected: string) => {
    const correction = {
      endMs: word.endMs,
      startMs: word.startMs,
      text: corrected,
    };
    pending.add(correction);
    onCorrect(correction);
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
      correct(word, word.heard);
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
    if (index === 0) return;
    const gap = { after: word, before: text.words[index - 1], durationMs };
    const kind = action === "clear-break" ? null : action;
    onChange({ breaks: withBreak(breaks, gap, kind) });
  };
  const openMenu = useCaptionMenu(act);

  if (!text) return null;
  return (
    <div className="flex flex-col gap-control">
      <CaptionTextView
        editing={editing}
        isDisabled={isDisabled}
        marks={marks}
        onEditEnd={(index, typed) => {
          setEditing(null);
          if (typed === null) return;
          const word = text.words[index];
          const corrected = normalizeText(typed);
          // Nothing left is the word taken out of the captions, as Backspace
          // takes it; the recording keeps it.
          if (corrected === "") {
            if (text.isMarked && isShown(index)) act("hide", index);
            return;
          }
          if (corrected !== normalizeText(word.text)) correct(word, corrected);
        }}
        onEditStart={setEditing}
        onHideToggle={(word) => {
          act(isShown(word) ? "hide" : "show", word);
        }}
        onMenu={(word, bounds) => {
          void openMenu(
            word,
            boundsAnchor(bounds),
            captionMenuItems({
              gap: gapBefore(word),
              isCorrected: text.words[word].corrected,
              isShown: isShown(word),
            }),
          );
        }}
        onSeek={onSeek}
        playheadMs={playheadMs}
        text={text}
      />
      {/* Spelling is the transcript's, shared with the Transcript panel,
       * so only what was placed here goes. */}
      <div className="flex justify-end">
        <Button
          isDisabled={
            isDisabled || (breaks.length === 0 && choices.length === 0)
          }
          onPress={() => {
            onChange({ breaks: [], wordChoices: [] });
          }}
        >
          {t("editor-panels-captions-reset-text")}
        </Button>
      </div>
    </div>
  );
}
