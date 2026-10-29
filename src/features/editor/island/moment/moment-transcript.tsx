// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { Alert } from "../../../../primitives/alert/alert";
import { CircularProgress } from "../../../../primitives/circular-progress/circular-progress";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";
import { Text } from "../../../../primitives/text/text";
import { IslandModelLine } from "../island-model-line";

import { MOMENT_ISLAND_WIDTH } from "./moment-width";

import type { MomentTranscript } from "./moment-island";

// Under a voice note's player, at the foot of the island: what was said
// once it is known; until then a short line saying it is being worked out,
// an alert that it could not be, as the islands' failures are, or the
// model it needs first, fetched from right here as every island's models
// are. The black grows as the words arrive.

/** A short line about the transcript. */
function Note({ children }: { children: string }) {
  return (
    <Text as="p" className="text-fg-secondary" variant="caption">
      {children}
    </Text>
  );
}

/** A short line on the island's text column, the small inset in from the
 * section's. */
const SHORT_LINE = "flex items-center gap-inline px-inset-sm";

export function MomentTranscriptLine({
  onDownloadModel,
  transcript,
}: {
  onDownloadModel: () => void;
  transcript: MomentTranscript;
}) {
  switch (transcript.status) {
    // What was said keeps to the island's text column, the small inset in
    // from the part's edges, as its notes do.
    case "ready":
      return transcript.text.trim() ? (
        <ScrollArea
          className="px-inset-sm"
          constrainHeight
          rootClassName={cn("max-h-32", MOMENT_ISLAND_WIDTH.part)}
        >
          <Text as="p" className="select-text" variant="body">
            {transcript.text}
          </Text>
        </ScrollArea>
      ) : (
        <div className={SHORT_LINE}>
          <Note>{t("editor-island-moment-no-speech")}</Note>
        </div>
      );
    case "failed":
      return (
        <Alert
          className={cn("px-inset-sm", MOMENT_ISLAND_WIDTH.part)}
          color="error"
          role="status"
        >
          {t("editor-island-moment-transcribe-failed")}
        </Alert>
      );
    // Needing a model is a line across the island, as what was said will
    // be.
    case "noModel":
      return (
        <div className={MOMENT_ISLAND_WIDTH.part}>
          <IslandModelLine
            model={transcript.model}
            onDownload={onDownloadModel}
            title={t("editor-island-transcription")}
          />
        </div>
      );
    case "queued":
    case "transcribing":
      return (
        <div className={SHORT_LINE}>
          <CircularProgress
            aria-label={t("editor-island-transcribing")}
            isIndeterminate
            size="small"
          />
          <Note>
            {transcript.status === "transcribing"
              ? t("editor-island-transcribing")
              : t("editor-island-moment-waiting")}
          </Note>
        </div>
      );
  }
}
