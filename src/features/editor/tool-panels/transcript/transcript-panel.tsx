// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";

import { planRecordingSilences } from "../../recording/microphone/microphone-api";
import { EditorKind } from "../../types";
import { sceneOffer } from "../scene/scene-offer";
import { TranscriptRequest } from "../tool-panel-transcript";
import { forwardToolPanelShortcut } from "../use-panel-shortcuts";
import { useToolPanelSnapshot } from "../use-tool-panel-snapshot";

import { useTranscriptChannel } from "./transcript-channel";
import { TranscriptPanelView } from "./transcript-panel-view";
import {
  useTranscriptPlayheadFeed,
  useTranscriptWordsFeed,
} from "./use-transcript-feed";

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

  useTranscriptPlayheadFeed();
  useTranscriptWordsFeed(transcript);

  // What Remove silences would cut, read up front so hovering Remove can
  // show it: transcribing has already listened through the track, so the
  // plan comes back at once.
  useEffect(() => {
    useTranscriptChannel.setState({ plannedSilences: [] });
    const artifactId = transcript?.artifactId;
    if (transcript?.status !== "ready" || artifactId === undefined) return;
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
      onMove={(move) => {
        ask({ move, type: "move" });
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
      onScene={(target, choice) => {
        ask({ choice, target, type: "scene" });
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
      sceneOffer={
        snapshot.scene && !snapshot.isLocked
          ? sceneOffer(snapshot.scene, snapshot)
          : null
      }
      transcript={transcript}
      words={words}
    />
  );
}
