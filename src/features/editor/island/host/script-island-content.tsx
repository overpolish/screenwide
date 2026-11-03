// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { sceneOffer } from "../../tools/scene/scene-offer";
import { forwardToolPanelShortcut } from "../../tools/use-panel-shortcuts";

import type { ToolPanelPatch } from "../../tools/tool-panel-patch";
import type { ToolPanelSnapshot } from "../../tools/tool-panel-store";
import type {
  ToolPanelTranscript,
  TranscriptRange,
  TranscriptRequest,
} from "../../tools/tool-panel-transcript";
import type { TranscriptWord } from "../../tools/transcript/transcript-words";
import type { EditorKind } from "../../types";
import type { IslandTranscript } from "../island-transcript";
import type { ScriptTranscriptProps } from "../script/script-island-props";

/** The Script island on its own, the Transcript mode's, asking the editor
 * through `ToolPanelPatch`: every cut, move and correction a step of the
 * edit history, and undo, redo and playback the editor's own keys.
 * Corrections and seeking are the caller's, which the Captions mode's
 * script shares. */
export function scriptTranscriptProps({
  change,
  language,
  plannedSilences,
  playheadMs,
  snapshot,
  source,
  transcript,
  words,
  workspace,
}: {
  change: (values: ToolPanelPatch) => void;
  language: string | null;
  plannedSilences: readonly TranscriptRange[];
  playheadMs: number;
  snapshot: ToolPanelSnapshot;
  source: ToolPanelTranscript;
  transcript: IslandTranscript;
  words: readonly TranscriptWord[];
  workspace: EditorKind;
}): Omit<ScriptTranscriptProps, "onCorrect" | "onSeek"> & {
  mode: "transcript";
} {
  const ask = (transcriptRequest: TranscriptRequest) => {
    change({ transcriptRequest });
  };
  return {
    isLocked: snapshot.isLocked,
    language,
    mode: "transcript",
    onCut: (ranges) => {
      ask({ ranges, type: "cut" });
    },
    onDownloadModel: () => {
      ask({ type: "download" });
    },
    onHighlight: (range) => {
      ask({ range, type: "highlight" });
    },
    onMove: (move) => {
      ask({ move, type: "move" });
    },
    onNotProfanity: (text) => {
      ask({ text, type: "not-profanity" });
    },
    onRedo: () => {
      forwardToolPanelShortcut(workspace, "KeyZ", {
        ctrlKey: true,
        key: "z",
        shiftKey: true,
      });
    },
    onRemoveSilences: () => {
      ask({ type: "remove-silences" });
    },
    onRestore: (ranges) => {
      ask({ ranges, type: "restore" });
    },
    onRestoreAll: () => {
      ask({ type: "restore-all" });
    },
    onScene: (target, choice) => {
      ask({ choice, target, type: "scene" });
    },
    onTogglePlayback: () => {
      forwardToolPanelShortcut(workspace, "KeyP");
    },
    onTranscribe: () => {
      ask({ type: "transcribe" });
    },
    onUndo: () => {
      forwardToolPanelShortcut(workspace, "KeyZ", { ctrlKey: true, key: "z" });
    },
    plannedSilences,
    playheadMs,
    sceneOffer:
      snapshot.scene && !snapshot.isLocked
        ? sceneOffer(snapshot.scene, snapshot)
        : null,
    script: {
      allowedWords: source.allowedWords,
      canRestore: source.canRestore,
      corrections: source.corrections,
      durationMs: source.durationMs,
      played: source.played,
      removed: source.removed,
      scenes: source.scenes,
      silences: source.silences,
    },
    transcript,
    words,
  };
}
