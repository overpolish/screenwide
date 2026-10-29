// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { Alert } from "../../../../primitives/alert/alert";
import { Text } from "../../../../primitives/text/text";
import { IslandModelLine } from "../island-model-line";
import { IslandWorkLine } from "../island-work-line";

import type { CaptionsTranscript } from "./captions-island";

/** Until the captions have a transcript to be made of: the model it needs,
 * fetched from right here as every island's models are; how far
 * transcribing has got; an alert that it could not be made, as the
 * islands' failures are; or a note that nothing was said, so there is
 * nothing to caption. */
export function CaptionsTranscriptLine({
  onDownloadModel,
  transcript,
}: {
  onDownloadModel: () => void;
  transcript: Exclude<CaptionsTranscript, { status: "ready" }>;
}) {
  switch (transcript.status) {
    case "noModel":
      return (
        <IslandModelLine
          model={transcript.model}
          onDownload={onDownloadModel}
          title={t("editor-island-transcription")}
        />
      );
    case "transcribing":
      return (
        <IslandWorkLine
          label={t("editor-island-captions-transcribing")}
          value={transcript.progress}
        />
      );
    case "failed":
      return (
        <Alert className="px-inset-sm" color="error" role="status">
          {t("editor-island-captions-transcribe-failed")}
        </Alert>
      );
    case "noSpeech":
      return (
        <Text
          as="p"
          className="px-inset-sm text-fg-secondary"
          variant="caption"
        >
          {t("editor-island-captions-no-speech")}
        </Text>
      );
  }
}
