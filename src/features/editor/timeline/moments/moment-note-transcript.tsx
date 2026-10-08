// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { Button } from "../../../../components/base/button/button";
import { CircularProgress } from "../../../../components/base/circular-progress/circular-progress";
import { ProgressBar } from "../../../../components/base/progress-bar/progress-bar";
import { ScrollArea } from "../../../../components/base/scroll-area/scroll-area";
import { Text } from "../../../../components/base/text/text";
import { t } from "../../../../i18n/i18n";
import {
  downloadTranscriptionModel,
  listenToTranscription,
} from "../../../transcription/api";

import type { NoteTranscript } from "./recording-moments";
import type { TranscriptionModel } from "../../../transcription/types";

/**
 * No model to transcribe the note with: the model it needs, downloaded from
 * right here rather than from Settings, with its progress as it comes down.
 */
function NoModel({ model }: { model: string }) {
  const [download, setDownload] = useState<TranscriptionModel | null>(null);

  useEffect(() => {
    let stopped = false;
    let stop: (() => void) | undefined;
    void listenToTranscription((state) => {
      setDownload(state.models.find((each) => each.id === model) ?? null);
    })
      .then((unlisten) => {
        if (stopped) unlisten();
        else stop = unlisten;
      })
      .catch(() => undefined);
    return () => {
      stopped = true;
      stop?.();
    };
  }, [model]);

  if (download?.status === "downloading")
    return (
      <div className="gap-control flex items-center">
        <Text className="shrink-0" variant="footnote">
          {t("editor-timeline-downloading-model")}
        </Text>
        <ProgressBar
          aria-label={t("editor-timeline-downloading-model-label")}
          value={(download.progress ?? 0) * 100}
        />
      </div>
    );
  return (
    <div className="gap-control flex items-center justify-between">
      <Text variant="footnote">{t("editor-timeline-no-model")}</Text>
      <Button
        onPress={() => {
          void downloadTranscriptionModel(model).catch((error: unknown) => {
            console.error("Could not download the transcription model", error);
          });
        }}
      >
        {t("editor-timeline-download")}
      </Button>
    </div>
  );
}

/**
 * Under a voice note's waveform: what was said, that it is being worked
 * out, that it could not be, or that a model is needed before it can be.
 */
export function MomentNoteTranscript({
  transcript,
}: {
  transcript: NoteTranscript;
}) {
  if (transcript.status === "ready" && !transcript.text.trim())
    return <Text variant="footnote">{t("editor-timeline-no-speech")}</Text>;
  if (transcript.status === "ready")
    return (
      <ScrollArea
        // Room for the scrollbar, so it never sits over the last word.
        className="pr-control-inset"
        constrainHeight
        edgeEffect="shadow"
        rootClassName="max-h-32"
      >
        <Text className="select-text">{transcript.text}</Text>
      </ScrollArea>
    );
  if (transcript.status === "failed")
    return (
      <Text variant="footnote">{t("editor-timeline-transcribe-failed")}</Text>
    );
  if (transcript.status === "noModel")
    return <NoModel model={transcript.model} />;
  return (
    <div className="gap-control flex items-center justify-center">
      <CircularProgress
        aria-label={t("editor-timeline-transcribing")}
        isIndeterminate
        size="small"
      />
      <Text variant="footnote">
        {transcript.status === "transcribing"
          ? t("editor-timeline-transcribing")
          : t("editor-timeline-transcribe-waiting")}
      </Text>
    </div>
  );
}
