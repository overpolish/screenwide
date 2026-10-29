// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { DEFAULT_CAPTION_SETTINGS } from "../../recording/captions/recording-captions";
import {
  STORY_TEXT_TEMPLATES,
  storyTextStyle,
} from "../../tool-panels/text-style/text-template-fixtures";
import { sampleTranscriptsModel } from "../../tool-panels/transcript/transcript-panel-fixtures";
import { captionsIsland } from "../captions/captions-island";
import {
  EditorIslandPairStage,
  EditorIslandStage,
} from "../editor-island-stage";
import {
  editorIslandStoryMeta,
  type EditorIslandStoryProps,
} from "../editor-island-story-meta";

import { scriptIsland } from "./script-island";
import {
  LONG_SCRIPT_TAKE,
  SCRIPT_TAKE,
  storyScriptText,
} from "./script-story-fixtures";
import {
  ScriptTranscriptStory,
  type ScriptTranscriptStoryActions,
} from "./script-transcript-story";

import type { RecordingCaptions } from "../../../../bindings/RecordingCaptions";
import type { CaptionsPatch } from "../../recording/captions/recording-captions";
import type { TranscriptCorrection } from "../../tool-panels/tool-panel-transcript";
import type { IslandTranscript } from "../island-transcript";
import type { Meta, StoryObj } from "@storybook/react-vite";

type ScriptStoryProps = EditorIslandStoryProps &
  ScriptTranscriptStoryActions & {
    captions: RecordingCaptions;
    /** Spelling put right in the transcript. */
    corrections: TranscriptCorrection[];
    /** Whether it stands beside the Captions island, as the Captions mode
     * opens the two. */
    isBesideCaptions: boolean;
    /** Whether a save is running, so the script can be read and not
     * edited. */
    isLocked: boolean;
    /** Whether the take is long enough to scroll within the island. */
    isLong: boolean;
    /** Whether the captions have been made yet. */
    isMade: boolean;
    /** Beside the captions, or on its own to cut. */
    mode: "captions" | "transcript";
    onCaptionsChange: (patch: CaptionsPatch) => void;
    playheadMs: number;
    /** Where the transcript stands, on its own. */
    transcript: IslandTranscript;
  };

const CAPTIONS: RecordingCaptions = {
  enabled: true,
  settings: DEFAULT_CAPTION_SETTINGS,
  style: storyTextStyle("box"),
};

function ScriptIsland({
  isBesideCaptions,
  isLong,
  isMade,
  isShown = true,
  mode,
  ...props
}: ScriptStoryProps) {
  if (mode === "transcript")
    return (
      <ScriptTranscriptStory
        actions={props}
        isLocked={props.isLocked}
        isShown={isShown}
        transcript={props.transcript}
      />
    );
  const take = isLong ? LONG_SCRIPT_TAKE : SCRIPT_TAKE;
  const script = scriptIsland({
    ...props,
    durationMs: take.durationMs,
    marks: take.marks,
    mode,
    text: storyScriptText(props.captions, props.corrections, { isMade, take }),
  });
  if (!isBesideCaptions)
    return <EditorIslandStage content={script} isShown={isShown} />;
  const { captions } = props;
  const change = (patch: CaptionsPatch) => {
    props.onCaptionsChange(patch);
  };
  return (
    <EditorIslandPairStage
      end={script}
      isShown={isShown}
      start={captionsIsland({
        canReset: false,
        canSetDefault: false,
        enabled: captions.enabled,
        hiddenCount: 0,
        look: captions.style,
        onDownloadModel: fn(),
        onEnabledChange: (enabled) => {
          change({ enabled });
        },
        onLayoutChange: (layout) => {
          change({ style: { ...captions.style, ...layout } });
        },
        onLookChange: (look) => {
          change({ style: { ...captions.style, ...look } });
        },
        onRemoveColor: fn(),
        onReset: fn(),
        onRestoreHidden: fn(),
        onSaveColor: fn(),
        onSetDefault: fn(),
        onSettingsChange: (settings) => {
          change({ settings: { ...captions.settings, ...settings } });
        },
        savedColors: [],
        settings: captions.settings,
        templates: STORY_TEXT_TEMPLATES,
        transcript: { status: "ready" },
      })}
    />
  );
}

const meta = {
  ...editorIslandStoryMeta,
  args: {
    captions: CAPTIONS,
    corrections: [],
    isBesideCaptions: false,
    isLocked: false,
    isLong: false,
    isMade: true,
    isShown: true,
    mode: "captions",
    onCaptionsChange: fn(),
    onCorrect: fn(),
    onCut: fn(),
    onDownloadModel: fn(),
    onMove: fn(),
    onRemoveSilences: fn(),
    onRestore: fn(),
    onRestoreAll: fn(),
    onSeek: fn(),
    onTranscribe: fn(),
    playheadMs: 5_200,
    transcript: { status: "ready" },
  },
  component: ScriptIsland,
  decorators: [
    featureStage(({ args }) =>
      args.isBesideCaptions
        ? { height: 760, width: 1280 }
        : { height: args.mode === "transcript" ? 760 : 640, width: 520 },
    ),
  ],
  // Every edit changes the story's script, as it would the recording's: a
  // break, a hidden word or a key word remakes the captions, a correction
  // goes into the transcript, and a press moves the playhead.
  render: function Render(args) {
    const [, updateArgs] = useArgs<ScriptStoryProps>();
    return (
      <ScriptIsland
        {...args}
        onCaptionsChange={(patch) => {
          args.onCaptionsChange(patch);
          const { captions } = args;
          updateArgs({
            captions: {
              ...captions,
              ...patch,
              settings: { ...captions.settings, ...patch.settings },
              style: { ...captions.style, ...patch.style },
            },
          });
        }}
        onCorrect={(corrections) => {
          args.onCorrect(corrections);
          const replaces = (made: TranscriptCorrection) =>
            corrections.some(
              (correction) =>
                made.startMs === correction.startMs &&
                made.endMs === correction.endMs,
            );
          updateArgs({
            corrections: [
              ...args.corrections.filter((made) => !replaces(made)),
              ...corrections,
            ],
          });
        }}
        onSeek={(playheadMs) => {
          args.onSeek(playheadMs);
          updateArgs({ playheadMs });
        }}
      />
    );
  },
  title: "Features/Editor/Island/Script",
} satisfies Meta<typeof ScriptIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Captions: Story = {};

// Beside the Captions island, as the Captions mode opens the two: a rule
// turned on there, such as hiding filler words, shows in the script.
export const BesideCaptions: Story = { args: { isBesideCaptions: true } };

// A template that sets a key word apart: each caption's reads bold.
export const KeyWords: Story = {
  args: { captions: { ...CAPTIONS, style: storyTextStyle("supersize") } },
};

// Before the captions are made, nothing is marked and every word shows
// plain.
export const BeforeCaptionsAreMade: Story = { args: { isMade: false } };

// A take long enough that the script scrolls within the island, faded at
// the edge where there is more.
export const LongScript: Story = { args: { isLong: true } };

// On its own, to cut, as the Transcript mode opens it. Choose words by
// clicking or dragging, hold Command (Ctrl) to remove or restore them,
// drag chosen words to move them, double-click to correct, Space to play.
// Hover a Remove to see what it would take.
export const Transcript: Story = { args: { mode: "transcript" } };

// A save is running: the script can be read and words chosen, not edited.
export const TranscriptWhileSaving: Story = {
  args: { isLocked: true, mode: "transcript" },
};

// The transcription model is not on this computer yet.
export const TranscriptModelToDownload: Story = {
  args: {
    mode: "transcript",
    transcript: {
      model: sampleTranscriptsModel("available"),
      status: "noModel",
    },
  },
};

export const Transcribing: Story = {
  args: {
    mode: "transcript",
    transcript: { progress: 0.35, status: "transcribing" },
  },
};
