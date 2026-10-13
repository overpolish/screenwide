// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { useTranscriptRevision } from "../transcript/transcript-revision";

import { makeRecordingCaptions } from "./captions-api";

import type { Caption } from "../../../../bindings/Caption";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

const NONE: Caption[] = [];

/**
 * The captions the edit asks of the recording, set in the output canvas as
 * the picture shows them. Made again whenever the edit, the canvas or the
 * transcript changes, which also hands them to the preview to draw - or
 * takes them away, while they are off. Until a new answer arrives, the lane
 * keeps the last one.
 */
export function useRecordingCaptions({
  artifactId,
  edit,
  output,
}: {
  artifactId: number;
  edit: RecordingTimelineEdit;
  output: { height: number; width: number } | undefined;
}) {
  const [captions, setCaptions] = useState<Caption[]>(NONE);
  const transcript = useTranscriptRevision();
  const hasTranscript =
    transcript.artifactId === artifactId && transcript.isReady;
  const isShown = edit.captions?.enabled === true && hasTranscript;
  const height = output?.height ?? 1;
  const width = output?.width ?? 1;
  const revision = transcript.revision;
  useEffect(() => {
    let current = true;
    makeRecordingCaptions(artifactId, edit, { height, width })
      .then((made) => {
        if (current) setCaptions(made ?? NONE);
      })
      .catch((cause: unknown) => {
        console.error("Could not make the captions", cause);
      });
    return () => {
      current = false;
    };
  }, [artifactId, edit, hasTranscript, height, revision, width]);
  return isShown ? captions : NONE;
}
