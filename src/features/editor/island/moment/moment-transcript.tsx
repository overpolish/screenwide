// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { Button } from "../../../../primitives/button/button";
import { CircularProgress } from "../../../../primitives/circular-progress/circular-progress";
import { ProgressBar } from "../../../../primitives/progress-bar/progress-bar";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";
import { Text } from "../../../../primitives/text/text";
import { EDITOR_ISLAND_WIDTH } from "../editor-island-width";

import type { MomentTranscript } from "./moment-island";

// Under a voice note's player, at the foot of the island: what was said
// once it is known; until then a short line saying it is being worked out,
// that it could not be, or that a model is needed first, fetched from right
// here rather than from Settings. The black grows as the words arrive.

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
          rootClassName={cn("max-h-32", EDITOR_ISLAND_WIDTH.part)}
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
        <div className={SHORT_LINE}>
          <Note>{t("editor-island-moment-transcribe-failed")}</Note>
        </div>
      );
    // Needing a model is a line across the island, as what was said will
    // be: what is needed at the start, and the download or its progress at
    // the end.
    case "noModel":
      return transcript.download === null ? (
        <div
          className={cn(
            "flex items-center justify-between gap-separate ps-inset-sm",
            EDITOR_ISLAND_WIDTH.part,
          )}
        >
          <Note>{t("editor-island-moment-no-model")}</Note>
          <Button onPress={onDownloadModel}>
            {t("editor-island-moment-download")}
          </Button>
        </div>
      ) : (
        <div
          className={cn(
            "flex items-center justify-between gap-separate px-inset-sm",
            EDITOR_ISLAND_WIDTH.part,
          )}
        >
          <Note>{t("editor-island-moment-downloading")}</Note>
          <ProgressBar
            aria-label={t("editor-island-moment-downloading-label")}
            className="w-24"
            value={transcript.download * 100}
          />
        </div>
      );
    case "queued":
    case "transcribing":
      return (
        <div className={SHORT_LINE}>
          <CircularProgress
            aria-label={t("editor-island-moment-transcribing")}
            isIndeterminate
            size="small"
          />
          <Note>
            {transcript.status === "transcribing"
              ? t("editor-island-moment-transcribing")
              : t("editor-island-moment-waiting")}
          </Note>
        </div>
      );
  }
}
