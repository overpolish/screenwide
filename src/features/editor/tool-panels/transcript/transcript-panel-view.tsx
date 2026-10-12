// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "../../../../components/base/button/button";
import { Text } from "../../../../components/base/text/text";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { ModelDownloadRow } from "../selection/microphone-switch-rows";
import { ToolProgress } from "../selection/tool-progress";
import {
  ToolPanelTranscript,
  TranscriptCorrection,
  TranscriptRange,
} from "../tool-panel-transcript";

import { TranscriptEditor } from "./transcript-editor";
import { TranscriptWord } from "./transcript-words";

export type TranscriptPanelViewProps = {
  isLocked: boolean;
  /** The language the transcriber heard, as its code; null until read. */
  language: string | null;
  onCancel: () => void;
  /** Give the words this stretch covers this text. */
  onCorrect: (correction: TranscriptCorrection) => void;
  /** Apply every correction in one undo step, for Replace all. */
  onCorrectMany: (corrections: TranscriptCorrection[]) => void;
  /** Cut these stretches, each one run of words still kept. */
  onCut: (ranges: TranscriptRange[]) => void;
  onDownload: () => void;
  /** Show this stretch on the editor's timeline as the words chosen, or
   * none with null. */
  onHighlight: (range: TranscriptRange | null) => void;
  /** Stop marking `text`, folded, as profanity anywhere in the recording. */
  onNotProfanity: (text: string) => void;
  onRedo: () => void;
  /** Run Remove silences, the microphone's own. */
  onRemoveSilences: () => void;
  /** Bring back these stretches, each one run of struck words. */
  onRestore: (ranges: TranscriptRange[]) => void;
  /** Bring back everything the panel cut, and every silence Remove
   * silences cut. */
  onRestoreAll: () => void;
  onSeek: (ms: number) => void;
  onTogglePlayback: () => void;
  onTranscribe: () => void;
  onUndo: () => void;
  /** What Remove silences would cut, so hovering Remove can show which
   * pauses it acts on. */
  plannedSilences: readonly TranscriptRange[];
  playheadMs: number;
  /** Null until the editor has published one. */
  transcript: ToolPanelTranscript | null;
  /** Null until the transcript has been read. */
  words: readonly TranscriptWord[] | null;
};

/**
 * The Transcript tool's panel: the recording's microphone as words, edited
 * in place. Before there are words, the one step that gets them: the
 * model's download, then transcribing, which can be cancelled. Nothing
 * until the editor has said where the transcript is.
 */
export function TranscriptPanelView(props: TranscriptPanelViewProps) {
  const { isLocked, transcript } = props;
  const model = transcript?.model ?? null;
  if (!transcript || !model) return null;

  if (model.status !== "downloaded")
    return (
      <ModelDownloadRow
        description={t("editor-panels-transcript-description")}
        isLocked={isLocked}
        model={model}
        onDownload={props.onDownload}
        title={t("editor-panels-transcript")}
      />
    );

  if (transcript.status === "ready")
    return props.words ? (
      <TranscriptEditor
        {...props}
        transcript={transcript}
        words={props.words}
      />
    ) : null;

  const isTranscribing = transcript.status === "transcribing";
  return (
    <div>
      <ControlRow
        description={t("editor-panels-transcript-transcribe-description")}
        title={t("editor-panels-transcript")}
      >
        {(controlProps) =>
          isTranscribing ? (
            <Button {...controlProps} onPress={props.onCancel}>
              {t("editor-panels-transcript-cancel")}
            </Button>
          ) : (
            <Button
              {...controlProps}
              isDisabled={isLocked}
              onPress={props.onTranscribe}
            >
              {t("editor-panels-transcript-transcribe")}
            </Button>
          )
        }
      </ControlRow>
      <ToolProgress
        isWorking={isTranscribing}
        label={t("editor-panels-transcript-working")}
        value={transcript.progress}
      />
      {transcript.error ? (
        <Text className="mt-control text-error" variant="footnote">
          {transcript.error}
        </Text>
      ) : null}
    </div>
  );
}
