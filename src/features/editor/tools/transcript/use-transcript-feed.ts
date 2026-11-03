// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";

import {
  getRecordingTranscript,
  TRANSCRIPT_PLAYHEAD_EVENT,
  TranscriptPlayhead,
} from "../../recording/transcript/transcript-api";
import { ToolPanelTranscript } from "../tool-panel-transcript";

import { useTranscriptChannel } from "./transcript-channel";

/** Follows the playhead into the channel while a panel that shows it is
 * open. It travels from the editor window on its own, so it never rides
 * the settings snapshot. */
export function useTranscriptPlayheadFeed() {
  useEffect(() => {
    let disposed = false;
    let stop: (() => void) | undefined;
    listen<TranscriptPlayhead>(TRANSCRIPT_PLAYHEAD_EVENT, (event) => {
      if (!disposed)
        useTranscriptChannel.setState({ playheadMs: event.payload.playheadMs });
    })
      .then((unlisten) => {
        if (disposed) unlisten();
        else stop = unlisten;
      })
      .catch((cause: unknown) => {
        console.error("Could not follow the transcript's playhead", cause);
      });
    return () => {
      disposed = true;
      stop?.();
    };
  }, []);
}

/** Reads the transcript's words and language into the channel, rather than
 * the snapshot, so a long recording's thousands never ride every change of
 * a setting. They are read again when the transcript is re-made, and
 * cleared when the recording changes. */
export function useTranscriptWordsFeed(transcript: ToolPanelTranscript | null) {
  const artifactId = transcript?.artifactId;
  const isReady = transcript?.status === "ready";
  const revision = transcript?.revision;
  useEffect(() => {
    useTranscriptChannel.setState({ language: null, words: null });
    if (!isReady || artifactId === undefined) return;
    let current = true;
    getRecordingTranscript(artifactId)
      .then((result) => {
        if (current)
          useTranscriptChannel.setState({
            language: result?.language ?? null,
            words: result?.words ?? null,
          });
      })
      .catch((cause: unknown) => {
        console.error("Could not read the transcript's words", cause);
      });
    return () => {
      current = false;
    };
  }, [artifactId, isReady, revision]);
}
