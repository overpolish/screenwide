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
    corrections: TranscriptCorrection[];
    // The Captions island out too, at the opposite corner.
    hasCaptions: boolean;
    // A save is running: the script can be read, not edited.
    isLocked: boolean;
    // A take long enough to scroll within the island.
    isLong: boolean;
    isMade: boolean;
    mode: "captions" | "transcript";
    onCaptionsChange: (patch: CaptionsPatch) => void;
    playheadMs: number;
    transcript: IslandTranscript;
  };

const CAPTIONS: RecordingCaptions = {
  enabled: true,
  settings: DEFAULT_CAPTION_SETTINGS,
  style: storyTextStyle("box"),
};

// The Captions island out with the Script island in the Captions mode.
const pairedCaptions = ({
  captions,
  onCaptionsChange,
  transcript,
}: Pick<ScriptStoryProps, "captions" | "onCaptionsChange" | "transcript">) =>
  captionsIsland({
    canReset: false,
    canSetDefault: false,
    enabled: captions.enabled,
    hiddenCount: 0,
    look: captions.style,
    onEnabledChange: (enabled) => {
      onCaptionsChange({ enabled });
    },
    onLayoutChange: (layout) => {
      onCaptionsChange({ style: { ...captions.style, ...layout } });
    },
    onLookChange: (look) => {
      onCaptionsChange({ style: { ...captions.style, ...look } });
    },
    onRemoveColor: fn(),
    onReset: fn(),
    onRestoreHidden: fn(),
    onSaveColor: fn(),
    onSetDefault: fn(),
    onSettingsChange: (settings) => {
      onCaptionsChange({ settings: { ...captions.settings, ...settings } });
    },
    savedColors: [],
    settings: captions.settings,
    templates: STORY_TEXT_TEMPLATES,
    transcript,
  });

function ScriptIsland({
  hasCaptions,
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
        corner={hasCaptions ? pairedCaptions(props) : undefined}
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
  if (!hasCaptions)
    return <EditorIslandStage content={script} isShown={isShown} />;
  return (
    <EditorIslandPairStage
      end={pairedCaptions(props)}
      isShown={isShown}
      start={script}
    />
  );
}

const meta = {
  ...editorIslandStoryMeta,
  args: {
    captions: CAPTIONS,
    corrections: [],
    hasCaptions: false,
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
      args.hasCaptions
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

// A rule turned on in the Captions island, such as hiding filler words,
// shows in the script.
export const WithCaptions: Story = { args: { hasCaptions: true } };

// The Captions island is only its row until the words are in, so moving
// between Transcript and Captions leaves the Script where it was.
export const WithCaptionsWhileTranscribing: Story = {
  args: {
    hasCaptions: true,
    mode: "transcript",
    transcript: { progress: 0.35, status: "transcribing" },
  },
};

export const KeyWords: Story = {
  args: { captions: { ...CAPTIONS, style: storyTextStyle("supersize") } },
};

// Before the captions are made, nothing is marked and every word shows
// plain.
export const BeforeCaptionsAreMade: Story = { args: { isMade: false } };

export const LongScript: Story = { args: { isLong: true } };

// Choose words by clicking or dragging, hold Command (Ctrl) to remove or
// restore them, drag to move them, double-click to correct, Space to play.
export const Transcript: Story = { args: { mode: "transcript" } };

export const TranscriptWhileSaving: Story = {
  args: { isLocked: true, mode: "transcript" },
};

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
