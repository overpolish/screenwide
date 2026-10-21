// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { ModelDownloadRow } from "../selection/microphone-switch-rows";
import { ToolProgress } from "../selection/tool-progress";
import { ToolPanelTranscript } from "../tool-panel-transcript";

/**
 * What a panel that works from the transcript shows while there is none
 * yet: the model to download, then Transcribe and how far it has got. The
 * Transcript and Captions panels both open with it; `description` says what
 * the transcript is for in the panel showing it.
 */
export function TranscriptSetup({
  description,
  isLocked,
  onCancel,
  onDownload,
  onTranscribe,
  transcript,
}: {
  description: string;
  isLocked: boolean;
  onCancel: () => void;
  onDownload: () => void;
  onTranscribe: () => void;
  transcript: ToolPanelTranscript;
}) {
  const model = transcript.model;
  if (!model) return null;

  if (model.status !== "downloaded")
    return (
      <ModelDownloadRow
        description={description}
        isLocked={isLocked}
        model={model}
        onDownload={onDownload}
        title={t("editor-panels-transcript")}
      />
    );

  const isTranscribing = transcript.status === "transcribing";
  return (
    <div>
      <ControlRow title={t("editor-panels-transcript")}>
        {(controlProps) =>
          isTranscribing ? (
            <Button {...controlProps} onPress={onCancel}>
              {t("editor-panels-transcript-cancel")}
            </Button>
          ) : (
            <Button
              {...controlProps}
              isDisabled={isLocked}
              onPress={onTranscribe}
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
        <p className="text-caption text-fg-secondary mt-control text-error">
          {transcript.error}
        </p>
      ) : null}
    </div>
  );
}
