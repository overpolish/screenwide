// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useMemo, useState } from "react";

import {
  createRecordingTimelineEdit,
  RecordingTimelineEdit,
} from "../../timeline/editing/recording-timeline-edit";
import { moveRecordingTimelineFootage } from "../../timeline/editing/recording-timeline-footage-move";
import {
  TranscriptCorrection,
  TranscriptRange,
  TranscriptScene,
} from "../../tool-panels/tool-panel-transcript";
import {
  addRanges,
  playedOnward,
  sampleTranscriptDurationMs,
  sampleTranscriptSilences,
  sampleTranscriptWords,
  sampleWordRange,
  subtractRanges,
} from "../../tool-panels/transcript/transcript-panel-fixtures";
import {
  idleSilences,
  scenesAfterPick,
  storySceneOffer,
} from "../../tool-panels/transcript/transcript-panel-story-states";
import { correctionsWith } from "../../tool-panels/transcript/transcript-units";
import { EditorIslandStage } from "../editor-island-stage";

import { scriptIsland } from "./script-island";

import type { ScriptTranscriptProps } from "./script-island-props";

type Edit = {
  allowed: string[];
  corrections: TranscriptCorrection[];
  removed: TranscriptRange[];
  scenes: TranscriptScene[];
  /** The order the recording plays in, which moving words changes. */
  timeline: RecordingTimelineEdit;
};
type History = { future: Edit[]; past: Edit[]; present: Edit };

/** What the story's Actions panel logs: everything the island asks. */
export type ScriptTranscriptStoryActions = Pick<
  ScriptTranscriptProps,
  | "onCorrect"
  | "onCut"
  | "onDownloadModel"
  | "onMove"
  | "onRemoveSilences"
  | "onRestore"
  | "onRestoreAll"
  | "onSeek"
  | "onTranscribe"
>;

/**
 * Stands in for the editor in the Script island's stories: cuts, restores,
 * corrections and moves change the words, with undo and redo, and Space
 * plays through what is kept, in the order the moves leave, jumping the
 * cuts as the recording would. A filler is cut and a word corrected from
 * the start, to show both.
 */
export function ScriptTranscriptStory({
  actions,
  isLocked,
  isShown,
  transcript,
}: {
  actions: ScriptTranscriptStoryActions;
  isLocked: boolean;
  isShown: boolean;
  transcript: ScriptTranscriptProps["transcript"];
}) {
  const [history, setHistory] = useState<History>(() => ({
    future: [],
    past: [],
    present: {
      allowed: [],
      corrections: [{ ...sampleWordRange("recordings"), text: "Recordings" }],
      removed: [sampleWordRange("um,")],
      scenes: [],
      timeline: createRecordingTimelineEdit(1),
    },
  }));
  const [playheadMs, setPlayheadMs] = useState(0);
  const [isPlaying, setIsPlaying] = useState(false);
  const { allowed, corrections, removed, scenes, timeline } = history.present;
  const words = sampleTranscriptWords;
  const played = useMemo(
    () =>
      timeline.segments.map((segment) => ({
        endMs: segment.sourceEnd * sampleTranscriptDurationMs,
        startMs: segment.sourceStart * sampleTranscriptDurationMs,
      })),
    [timeline],
  );

  useEffect(() => {
    if (!isPlaying) return;
    let last = performance.now();
    let frame = requestAnimationFrame(function tick(now) {
      // Read before the update runs: React calls it later, after `last` moves.
      const elapsed = Math.max(now - last, 0);
      last = now;
      setPlayheadMs((ms) => playedOnward(ms, elapsed, { played, removed }));
      frame = requestAnimationFrame(tick);
    });
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [isPlaying, played, removed]);

  const commit = (change: Partial<Edit>) => {
    setHistory(({ past, present }) => ({
      future: [],
      past: [...past, present],
      present: { ...present, ...change },
    }));
  };
  const travel = (by: -1 | 1) => {
    setHistory((current) => {
      const to =
        by < 0 ? current.past[current.past.length - 1] : current.future[0];
      if (!(to as Edit | undefined)) return current;
      return by < 0
        ? {
            future: [current.present, ...current.future],
            past: current.past.slice(0, -1),
            present: to,
          }
        : {
            future: current.future.slice(1),
            past: [...current.past, current.present],
            present: to,
          };
    });
  };

  const content = scriptIsland({
    isLocked,
    language: "en",
    mode: "transcript",
    onCorrect: (many) => {
      actions.onCorrect(many);
      commit({
        corrections: many.reduce(
          (made, correction) => correctionsWith(words, made, correction),
          corrections,
        ),
      });
    },
    onCut: (ranges) => {
      actions.onCut(ranges);
      commit({ removed: addRanges(removed, ranges) });
    },
    onDownloadModel: actions.onDownloadModel,
    onHighlight: () => undefined,
    onMove: (move) => {
      actions.onMove(move);
      commit({
        timeline: moveRecordingTimelineFootage(
          timeline,
          move,
          sampleTranscriptDurationMs,
        ),
      });
    },
    onNotProfanity: (text) => {
      commit({ allowed: [...allowed, text] });
    },
    onRedo: () => {
      travel(1);
    },
    onRemoveSilences: () => {
      actions.onRemoveSilences();
      commit({ removed: addRanges(removed, sampleTranscriptSilences) });
    },
    onRestore: (ranges) => {
      actions.onRestore(ranges);
      commit({ removed: subtractRanges(removed, ranges) });
    },
    onRestoreAll: () => {
      actions.onRestoreAll();
      commit({ removed: [] });
    },
    onScene: (target) => {
      commit({ scenes: scenesAfterPick(scenes, target) });
    },
    onSeek: (ms) => {
      actions.onSeek(ms);
      setPlayheadMs(ms);
    },
    onTogglePlayback: () => {
      setIsPlaying((playing) => !playing);
    },
    onTranscribe: actions.onTranscribe,
    onUndo: () => {
      travel(-1);
    },
    plannedSilences: sampleTranscriptSilences,
    playheadMs,
    sceneOffer: isLocked ? null : storySceneOffer,
    script: {
      allowedWords: allowed,
      canRestore: removed.length > 0,
      corrections,
      durationMs: sampleTranscriptDurationMs,
      played,
      removed,
      scenes,
      silences: idleSilences,
    },
    transcript,
    words,
  });
  return <EditorIslandStage content={content} isShown={isShown} />;
}
