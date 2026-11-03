// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useMemo, useRef } from "react";

import { planRecordingSilences } from "../../recording/microphone/microphone-api";
import {
  captionText,
  type CaptionTextWord,
} from "../../tools/captions/caption-text";
import { useCaptionList } from "../../tools/captions/use-caption-list";
import { useTranscriptChannel } from "../../tools/transcript/transcript-channel";
import { usePendingCorrections } from "../../tools/transcript/use-pending-corrections";
import {
  useTranscriptPlayheadFeed,
  useTranscriptWordsFeed,
} from "../../tools/transcript/use-transcript-feed";
import { useTranscriptMarks } from "../../tools/transcript/use-transcript-marks";
import { scriptIsland } from "../script/script-island";

import { scriptTranscriptProps } from "./script-island-content";

import type { ToolPanelPatch } from "../../tools/tool-panel-patch";
import type { ToolPanelSnapshot } from "../../tools/tool-panel-store";
import type {
  ToolPanelTranscript,
  TranscriptCorrection,
} from "../../tools/tool-panel-transcript";
import type { TranscriptWord } from "../../tools/transcript/transcript-words";
import type { EditorKind } from "../../types";
import type { EditorIslandContent } from "../editor-island";
import type { IslandTranscript } from "../island-transcript";
import type { EditorIslandKind } from "./editor-island-kind";

// The recording's script, for the Captions and Transcript modes: its words
// and the playhead read from the editor through their own channel into this
// window, and only while one of those islands is up. The captions break the
// script as they were last made; a word corrected shows at once, before the
// editor answers.

const NO_WORDS: readonly CaptionTextWord[] = [];
const NOTHING_HEARD: readonly TranscriptWord[] = [];
const NO_CORRECTIONS: readonly TranscriptCorrection[] = [];
const NO_ALLOWED: readonly string[] = [];

/** Where the transcript stands, as the islands say it. With the model on
 * this computer and nothing transcribed yet, transcribing is about to
 * start. */
const islandTranscriptOf = (
  transcript: ToolPanelTranscript,
): IslandTranscript => {
  if (transcript.status === "transcribing")
    return { progress: transcript.progress, status: "transcribing" };
  if (transcript.error !== null) return { status: "failed" };
  if (transcript.status === "ready") return { status: "ready" };
  if (transcript.model && transcript.model.status !== "downloaded")
    return { model: transcript.model, status: "noModel" };
  return { progress: 0, status: "transcribing" };
};

export function useScriptIsland({
  change,
  island,
  snapshot,
  workspace,
}: {
  change: (values: ToolPanelPatch) => void;
  island: EditorIslandKind | null;
  snapshot: ToolPanelSnapshot;
  workspace: EditorKind;
}): {
  script: EditorIslandContent | null;
  transcript: IslandTranscript | null;
} {
  const isCaptions = island === "captions";
  const isOpen = isCaptions || island === "transcript";
  const source = isOpen ? (snapshot.transcript ?? null) : null;
  useTranscriptPlayheadFeed();
  useTranscriptWordsFeed(source);
  const words = useTranscriptChannel((state) => state.words);
  const language = useTranscriptChannel((state) => state.language);
  const playheadMs = useTranscriptChannel((state) => state.playheadMs);
  const plannedSilences = useTranscriptChannel(
    (state) => state.plannedSilences,
  );
  const made = useCaptionList(isCaptions ? source?.artifactId : undefined);
  const pending = usePendingCorrections(
    words ?? NOTHING_HEARD,
    source?.corrections ?? NO_CORRECTIONS,
  );
  const { captions } = snapshot;
  const keyWords = captions?.style.accent === "word";
  const text = useMemo(
    () =>
      isCaptions && source && words && captions
        ? captionText(
            words,
            {
              corrections: pending.corrections,
              durationMs: source.durationMs,
              played: source.played,
              removed: source.removed,
            },
            { breaks: captions.breaks ?? [], captions: made, keyWords },
          )
        : null,
    [captions, isCaptions, keyWords, made, pending.corrections, source, words],
  );
  const { marks } = useTranscriptMarks(
    text?.words ?? NO_WORDS,
    language,
    source?.allowedWords ?? NO_ALLOWED,
  );

  // What Remove silences would cut, read up front so hovering Remove can
  // show it: transcribing has already listened through the track.
  const artifactId = source?.artifactId;
  const isReady = source?.status === "ready";
  const revision = source?.revision;
  useEffect(() => {
    useTranscriptChannel.setState({ plannedSilences: [] });
    if (!isReady || artifactId === undefined || isCaptions) return;
    let current = true;
    planRecordingSilences(artifactId)
      .then((cuts) => {
        if (current) useTranscriptChannel.setState({ plannedSilences: cuts });
      })
      .catch((cause: unknown) => {
        console.error("Could not plan the silences to preview", cause);
      });
    return () => {
      current = false;
    };
  }, [artifactId, isCaptions, isReady, revision]);

  // The islands offer no Transcribe of their own before there is a
  // transcript: opening one with the model on this computer starts it, once
  // for each recording.
  const askedRef = useRef<number | null>(null);
  const canStart =
    source !== null &&
    source.status === "none" &&
    source.error === null &&
    source.model?.status === "downloaded" &&
    !snapshot.isLocked;
  useEffect(() => {
    if (
      !canStart ||
      artifactId === undefined ||
      askedRef.current === artifactId
    )
      return;
    askedRef.current = artifactId;
    change({ transcriptRequest: { type: "transcribe" } });
  });

  if (!source) return { script: null, transcript: null };
  const transcript = islandTranscriptOf(source);
  const correct = (corrections: TranscriptCorrection[]) => {
    corrections.forEach(pending.add);
    change({
      transcriptRequest:
        corrections.length === 1
          ? { correction: corrections[0], type: "correct" }
          : { corrections, type: "correct-many" },
    });
  };
  const seek = (ms: number) => {
    change({ transcriptRequest: { ms, type: "seek" } });
  };
  // Until the transcript's words are in, the Captions mode's Script says
  // how it is coming as the Transcript mode's does, the same island, so
  // moving between the modes leaves it as it was.
  if (isCaptions && transcript.status === "ready")
    return {
      script:
        captions && text
          ? scriptIsland({
              captions,
              durationMs: source.durationMs,
              marks,
              mode: "captions",
              onCaptionsChange: (patch) => {
                change({ captions: patch });
              },
              onCorrect: correct,
              onSeek: seek,
              playheadMs,
              text,
            })
          : null,
      transcript,
    };
  return {
    script: scriptIsland({
      ...scriptTranscriptProps({
        change,
        language,
        plannedSilences,
        playheadMs,
        snapshot,
        source,
        transcript,
        words: words ?? NOTHING_HEARD,
        workspace,
      }),
      onCorrect: correct,
      onSeek: seek,
    }),
    transcript,
  };
}
