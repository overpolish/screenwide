// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { FeatureStoryStage } from "../../../../storybook/feature-story-stage";
import { toolPanelWidth } from "../../../popup-panel/layout";
import { ToolPanelSurface } from "../tool-panel";
import {
  TranscriptCorrection,
  TranscriptRange,
} from "../tool-panel-transcript";

import {
  addRanges,
  sampleTranscriptDurationMs,
  sampleTranscriptSilences,
  sampleTranscriptsModel,
  sampleTranscriptWords,
  sampleWordRange,
  skipRemoved,
  subtractRanges,
} from "./transcript-panel-fixtures";
import {
  before,
  idleSilences,
  ignore,
  still,
} from "./transcript-panel-story-states";
import { TranscriptPanelView } from "./transcript-panel-view";
import { correctionsWith } from "./transcript-units";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  decorators: [
    (Story, context) => (
      <FeatureStoryStage viewMode={context.viewMode} width={toolPanelWidth}>
        <ToolPanelSurface>
          <Story />
        </ToolPanelSurface>
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Editor/Tool Panel/Transcript",
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

type Edit = {
  allowed: string[];
  corrections: TranscriptCorrection[];
  removed: TranscriptRange[];
};
type History = { future: Edit[]; past: Edit[]; present: Edit };

/**
 * Stands in for the editor: cuts, restores and corrections change the words,
 * with undo and redo, and Space plays through what is kept, jumping the cuts
 * as the recording would.
 */
function LiveTranscript({ isLocked = false }: { isLocked?: boolean }) {
  const [history, setHistory] = useState<History>(() => ({
    future: [],
    past: [],
    // One filler word cut and one word corrected already, to show both.
    present: {
      allowed: [],
      corrections: [{ ...sampleWordRange("recordings"), text: "Recordings" }],
      removed: [sampleWordRange("um,")],
    },
  }));
  const [playheadMs, setPlayheadMs] = useState(0);
  const [isPlaying, setIsPlaying] = useState(false);
  const { allowed, corrections, removed } = history.present;

  useEffect(() => {
    if (!isPlaying) return;
    let last = performance.now();
    let frame = requestAnimationFrame(function tick(now) {
      // Read before the update runs: React calls it later, after `last` moves.
      const elapsed = Math.max(now - last, 0);
      last = now;
      setPlayheadMs((ms) =>
        Math.min(
          skipRemoved(ms + elapsed, removed),
          sampleTranscriptDurationMs,
        ),
      );
      frame = requestAnimationFrame(tick);
    });
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [isPlaying, removed]);

  const commit = (change: Partial<Edit>) => {
    setHistory(({ past, present }) => ({
      future: [],
      past: [...past, present],
      present: { ...present, ...change },
    }));
  };

  return (
    <TranscriptPanelView
      isLocked={isLocked}
      language="en"
      onCancel={ignore}
      onCorrect={(correction) => {
        commit({
          corrections: correctionsWith(
            sampleTranscriptWords,
            corrections,
            correction,
          ),
        });
      }}
      onCorrectMany={(many) => {
        commit({
          corrections: many.reduce(
            (acc, correction) =>
              correctionsWith(sampleTranscriptWords, acc, correction),
            corrections,
          ),
        });
      }}
      onCut={(ranges) => {
        commit({ removed: addRanges(removed, ranges) });
      }}
      onDownload={ignore}
      onHighlight={ignore}
      onNotProfanity={(text) => {
        commit({ allowed: [...allowed, text] });
      }}
      onRedo={() => {
        setHistory((current) => {
          const next = current.future[0] as Edit | undefined;
          return next
            ? {
                future: current.future.slice(1),
                past: [...current.past, current.present],
                present: next,
              }
            : current;
        });
      }}
      onRemoveSilences={ignore}
      onRestore={(ranges) => {
        commit({ removed: subtractRanges(removed, ranges) });
      }}
      onRestoreAll={() => {
        commit({ removed: [] });
      }}
      onSeek={setPlayheadMs}
      onTogglePlayback={() => {
        setIsPlaying((playing) => !playing);
      }}
      onTranscribe={ignore}
      onUndo={() => {
        setHistory((current) => {
          const previous = current.past[current.past.length - 1] as
            Edit | undefined;
          return previous
            ? {
                future: [current.present, ...current.future],
                past: current.past.slice(0, -1),
                present: previous,
              }
            : current;
        });
      }}
      plannedSilences={sampleTranscriptSilences}
      playheadMs={playheadMs}
      transcript={{
        allowedWords: allowed,
        artifactId: 1,
        canRestore: removed.length > 0,
        corrections,
        durationMs: sampleTranscriptDurationMs,
        error: null,
        model: sampleTranscriptsModel("downloaded"),
        progress: 1,
        removed,
        revision: 0,
        silences: idleSilences,
        status: "ready",
      }}
      words={sampleTranscriptWords}
    />
  );
}

/** The model is not on this computer yet: the panel offers its download. */
export const Download: Story = {
  render: () => (
    <TranscriptPanelView
      {...still(before(sampleTranscriptsModel("available")))}
    />
  ),
};

export const Downloading: Story = {
  render: () => (
    <TranscriptPanelView
      {...still(before(sampleTranscriptsModel("downloading", 0.4)))}
    />
  ),
};

/** The model is here and the recording has not been transcribed. */
export const ReadyToTranscribe: Story = {
  render: () => (
    <TranscriptPanelView
      {...still(before(sampleTranscriptsModel("downloaded")))}
    />
  ),
};

/** Transcribing, with Cancel in place of Transcribe. */
export const Transcribing: Story = {
  render: () => (
    <TranscriptPanelView
      {...still(
        before(sampleTranscriptsModel("downloaded"), "transcribing", 0.35),
      )}
    />
  ),
};

/** Transcribing failed: what went wrong shows as a footnote under the
 * Transcribe button. */
export const TranscribeFailed: Story = {
  render: () => (
    <TranscriptPanelView
      {...still({
        ...before(sampleTranscriptsModel("downloaded")),
        error: "The transcription could not finish.",
      })}
    />
  ),
};

/** The words to edit. Click, Shift-click or drag to choose words, Delete to
 * cut them, double-click or Enter to correct them, Command-click to cut or
 * restore one, Space to play. The right-click menu is a window of its own,
 * so it opens only in the native Storybook. */
export const Transcript: Story = {
  render: () => <LiveTranscript />,
};

/** A save is running: the words can be read and selected, not edited. */
export const Locked: Story = {
  render: () => <LiveTranscript isLocked />,
};
