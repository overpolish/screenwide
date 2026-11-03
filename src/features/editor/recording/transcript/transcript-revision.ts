// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";
import { create } from "zustand";

import type { ToolPanelTranscript } from "../../tools/tool-panel-transcript";
import type { TranscriptWord } from "../../tools/transcript/transcript-words";

/**
 * Which transcript the editor holds of the recording open, published by the
 * Transcript tool, so what is made from its words elsewhere in the window -
 * the Captions lane - knows when to make itself again: when the transcript
 * is read, or made afresh - and whether it heard any words, without which
 * the title bar offers no Captions or Transcript.
 */
export const useTranscriptRevision = create<{
  artifactId: number | null;
  hasWords: boolean;
  isReady: boolean;
  revision: number;
}>()(() => ({
  artifactId: null,
  hasWords: false,
  isReady: false,
  revision: 0,
}));

/** Publishes the Transcript tool's transcript of `artifactId`. */
export function usePublishTranscriptRevision({
  artifactId,
  revision,
  status,
  words,
}: {
  artifactId: number | null;
  revision: number;
  status: ToolPanelTranscript["status"];
  words: readonly TranscriptWord[];
}) {
  const isReady = status === "ready";
  const hasWords = words.length > 0;
  useEffect(() => {
    useTranscriptRevision.setState({
      artifactId,
      hasWords,
      isReady,
      revision,
    });
  }, [artifactId, hasWords, isReady, revision]);
}
