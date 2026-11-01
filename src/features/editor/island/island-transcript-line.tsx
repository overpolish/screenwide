// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { Alert } from "../../../primitives/alert/alert";

import { IslandModelLine } from "./island-model-line";
import { IslandWorkLine } from "./island-work-line";

import type { IslandTranscript } from "./island-transcript";

/** Until there is a transcript: the model it needs, fetched from right here
 * as every island's models are; how far transcribing has got; or an alert
 * that it could not be made, as the islands' failures are. A transcript that
 * heard nothing has no island: the title bar no longer offers Captions or
 * Transcript. */
export function IslandTranscriptLine({
  onDownloadModel,
  transcript,
}: {
  onDownloadModel: () => void;
  transcript: Exclude<IslandTranscript, { status: "ready" }>;
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
          label={t("editor-island-transcribing")}
          value={transcript.progress}
        />
      );
    case "failed":
      return (
        <Alert className="px-inset-sm" color="error" role="status">
          {t("editor-island-transcribe-failed")}
        </Alert>
      );
  }
}
