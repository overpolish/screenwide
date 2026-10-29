// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo } from "react";

import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { CaptionsPatch } from "../../recording/captions/recording-captions";
import { ToolPanelTranscript } from "../tool-panel-transcript";
import { useTranscriptChannel } from "../transcript/transcript-channel";
import { TranscriptWord } from "../transcript/transcript-words";
import { usePendingCorrections } from "../transcript/use-pending-corrections";
import {
  useTranscriptPlayheadFeed,
  useTranscriptWordsFeed,
} from "../transcript/use-transcript-feed";
import { useTranscriptMarks } from "../transcript/use-transcript-marks";

import { captionText, CaptionTextWord } from "./caption-text";
import { CaptionTextView } from "./caption-text-view";
import { useCaptionList } from "./use-caption-list";
import { useCaptionWordEditing } from "./use-caption-word-editing";

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
  // A correction shows as soon as it is typed, before the editor answers.
  const pending = usePendingCorrections(
    words ?? NOTHING_HEARD,
    transcript.corrections,
  );
  const corrections = pending.corrections;
  const { durationMs, played, removed } = transcript;
  const keyWords = captions.style.accent === "word";
  const text = useMemo(
    () =>
      words
        ? captionText(
            words,
            { corrections, durationMs, played, removed },
            {
              breaks: captions.breaks ?? [],
              captions: made,
              keyWords,
            },
          )
        : null,
    [
      captions.breaks,
      corrections,
      durationMs,
      keyWords,
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
  const { hasPlaced, reset, ...wordEditing } = useCaptionWordEditing({
    captions,
    durationMs,
    isDisabled,
    marks,
    onChange,
    onCorrect: correct,
    text,
  });

  if (!text) return null;
  return (
    <div className="flex flex-col gap-control">
      <CaptionTextView
        {...wordEditing}
        isDisabled={isDisabled}
        marks={marks}
        onSeek={onSeek}
        playheadMs={playheadMs}
        text={text}
      />
      {/* Spelling is the transcript's, shared with the Transcript panel,
       * so only what was placed here goes. */}
      <div className="flex justify-end">
        <Button isDisabled={isDisabled || !hasPlaced} onPress={reset}>
          {t("editor-panels-captions-reset-text")}
        </Button>
      </div>
    </div>
  );
}
