// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";

import { planRecordingSilences } from "../../recording/microphone/microphone-api";
import {
  getRecordingTranscript,
  TRANSCRIPT_PLAYHEAD_EVENT,
  TranscriptPlayhead,
} from "../../recording/transcript/transcript-api";
import { EditorKind } from "../../types";
import { TranscriptRequest } from "../tool-panel-transcript";
import { forwardToolPanelShortcut } from "../use-panel-shortcuts";
import { useToolPanelSnapshot } from "../use-tool-panel-snapshot";

import { useTranscriptChannel } from "./transcript-channel";
import { TranscriptPanelView } from "./transcript-panel-view";

/** The Transcript panel in its window: the editor's snapshot says where the
 * transcript is, the channel carries its words and the playhead, and every
 * ask goes back to the editor. */
export function TranscriptPanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const playheadMs = useTranscriptChannel((state) => state.playheadMs);
  const words = useTranscriptChannel((state) => state.words);
  const language = useTranscriptChannel((state) => state.language);
  const plannedSilences = useTranscriptChannel(
    (state) => state.plannedSilences,
  );
  const transcript = snapshot.transcript ?? null;
  const ask = (transcriptRequest: TranscriptRequest) => {
    change({ transcriptRequest });
  };

  // The playhead travels from the editor window on its own, so it never
  // rides the settings snapshot.
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

  // The words, fetched here rather than in the snapshot so a long recording's
  // thousands never ride every change of a setting. They are refetched when
  // the transcript is re-made, and cleared when the recording changes. What
  // Remove silences would cut is read with them: transcribing has already
  // listened through the track, so the plan comes back at once.
  useEffect(() => {
    const artifactId = transcript?.artifactId;
    if (transcript?.status !== "ready" || artifactId === undefined) {
      useTranscriptChannel.setState({
        language: null,
        plannedSilences: [],
        words: null,
      });
      return;
    }
    let current = true;
    useTranscriptChannel.setState({
      language: null,
      plannedSilences: [],
      words: null,
    });
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
        if (current) useTranscriptChannel.setState({ words: null });
      });
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
  }, [transcript?.artifactId, transcript?.revision, transcript?.status]);

  return (
    <TranscriptPanelView
      isLocked={snapshot.isLocked}
      language={language}
      onCancel={() => {
        ask({ type: "cancel" });
      }}
      onCorrect={(correction) => {
        ask({ correction, type: "correct" });
      }}
      onCorrectMany={(corrections) => {
        ask({ corrections, type: "correct-many" });
      }}
      onCut={(ranges) => {
        ask({ ranges, type: "cut" });
      }}
      onDownload={() => {
        ask({ type: "download" });
      }}
      onHighlight={(range) => {
        ask({ range, type: "highlight" });
      }}
      onNotProfanity={(text) => {
        ask({ text, type: "not-profanity" });
      }}
      onRedo={() => {
        forwardToolPanelShortcut(workspace, "KeyZ", {
          ctrlKey: true,
          key: "z",
          shiftKey: true,
        });
      }}
      onRemoveSilences={() => {
        ask({ type: "remove-silences" });
      }}
      onRestore={(ranges) => {
        ask({ ranges, type: "restore" });
      }}
      onRestoreAll={() => {
        ask({ type: "restore-all" });
      }}
      onSeek={(ms) => {
        ask({ ms, type: "seek" });
      }}
      onTogglePlayback={() => {
        forwardToolPanelShortcut(workspace, "KeyP");
      }}
      onTranscribe={() => {
        ask({ type: "transcribe" });
      }}
      onUndo={() => {
        forwardToolPanelShortcut(workspace, "KeyZ", {
          ctrlKey: true,
          key: "z",
        });
      }}
      plannedSilences={plannedSilences}
      playheadMs={playheadMs}
      transcript={transcript}
      words={words}
    />
  );
}
