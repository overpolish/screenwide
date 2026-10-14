// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { useTranscriptRevision } from "../transcript/transcript-revision";

import { makeRecordingCaptions } from "./captions-api";

import type { Caption } from "../../../../bindings/Caption";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

const NONE: Caption[] = [];

/** What the captions are made of: the edit, in a canvas of a size. */
type Ask = {
  artifactId: number;
  edit: RecordingTimelineEdit;
  height: number;
  width: number;
};

/**
 * The captions the edit asks of the recording, set in the output canvas as
 * the picture shows them. Made again whenever the edit, the canvas or the
 * transcript changes, which also hands them to the preview to draw - or
 * takes them away, while they are off. Until a new answer arrives, the lane
 * keeps the last one.
 *
 * One making at a time: a slider dragged sends a change a step, faster than
 * the captions can be made, so the steps that arrive while one is being made
 * give way to the latest, made as soon as it finishes. The preview always
 * ends on the last change, and never shows an earlier one after it.
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
  const makingRef = useRef<{ next: Ask | null; running: boolean }>({
    next: null,
    running: false,
  });
  useEffect(() => {
    const queue = makingRef.current;
    queue.next = { artifactId, edit, height, width };
    if (queue.running) return;
    const makeNext = () => {
      const ask = queue.next;
      queue.next = null;
      if (!ask) {
        queue.running = false;
        return;
      }
      queue.running = true;
      makeRecordingCaptions(ask.artifactId, ask.edit, ask)
        .then((made) => {
          // A newer ask waiting means this answer is already out of date.
          if (!queue.next) setCaptions(made ?? NONE);
        })
        .catch((cause: unknown) => {
          console.error("Could not make the captions", cause);
        })
        .finally(makeNext);
    };
    makeNext();
  }, [artifactId, edit, hasTranscript, height, revision, width]);
  return isShown ? captions : NONE;
}
