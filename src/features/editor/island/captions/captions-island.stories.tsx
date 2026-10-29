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
import { EditorIslandStage } from "../editor-island-stage";
import {
  editorIslandStoryMeta,
  type EditorIslandStoryProps,
} from "../editor-island-story-meta";

import { captionsIsland, type CaptionsIslandProps } from "./captions-island";

import type { FontFamily } from "../../../../bindings/FontFamily";
import type { Meta, StoryObj } from "@storybook/react-vite";

function CaptionsIsland({
  isShown,
  ...props
}: CaptionsIslandProps & EditorIslandStoryProps) {
  return (
    <EditorIslandStage content={captionsIsland(props)} isShown={isShown} />
  );
}

/** A few families, as the system would offer them. */
const FAMILIES: FontFamily[] = [
  { italic: true, name: "Georgia", weights: [400, 700] },
  { italic: true, name: "Helvetica Neue", weights: [300, 400, 500, 700] },
  { italic: false, name: "Menlo", weights: [400, 700] },
];

/** The transcription model, about the size the app's default one is. */
const MODEL_BYTES = 874_000_000;

const meta = {
  ...editorIslandStoryMeta,
  args: {
    canReset: false,
    canSetDefault: false,
    enabled: true,
    families: FAMILIES,
    hiddenCount: 0,
    isShown: true,
    look: storyTextStyle("box"),
    onDownloadModel: fn(),
    onEnabledChange: fn(),
    onLayoutChange: fn(),
    onLookChange: fn(),
    onRemoveColor: fn(),
    onReset: fn(),
    onRestoreHidden: fn(),
    onSaveColor: fn(),
    onSetDefault: fn(),
    onSettingsChange: fn(),
    savedColors: [],
    settings: DEFAULT_CAPTION_SETTINGS,
    templates: STORY_TEXT_TEMPLATES,
    transcript: { status: "ready" },
  },
  component: CaptionsIsland,
  decorators: [featureStage({ height: 720, width: 640 })],
  // Every choice changes the story's args, as it would the captions; a
  // change makes Reset and Set as default worth offering.
  render: function Render(args) {
    const [, updateArgs] = useArgs<CaptionsIslandProps>();
    const changed = { canReset: true, canSetDefault: true };
    return (
      <CaptionsIsland
        {...args}
        onEnabledChange={(enabled) => {
          args.onEnabledChange(enabled);
          updateArgs({ enabled });
        }}
        onLayoutChange={(layout) => {
          args.onLayoutChange(layout);
          updateArgs({ ...changed, look: { ...args.look, ...layout } });
        }}
        onLookChange={(patch) => {
          args.onLookChange(patch);
          updateArgs({ ...changed, look: { ...args.look, ...patch } });
        }}
        onRemoveColor={(color) => {
          args.onRemoveColor(color);
          updateArgs({
            savedColors: args.savedColors.filter((kept) => kept !== color),
          });
        }}
        onReset={() => {
          args.onReset();
          updateArgs({
            canReset: false,
            look: storyTextStyle(args.look.template),
            settings: DEFAULT_CAPTION_SETTINGS,
          });
        }}
        onRestoreHidden={() => {
          args.onRestoreHidden();
          updateArgs({ hiddenCount: 0 });
        }}
        onSaveColor={(color) => {
          args.onSaveColor(color);
          if (!args.savedColors.includes(color))
            updateArgs({ savedColors: [...args.savedColors, color] });
        }}
        onSetDefault={() => {
          args.onSetDefault();
          updateArgs({ canSetDefault: false });
        }}
        onSettingsChange={(settings) => {
          args.onSettingsChange(settings);
          updateArgs({
            ...changed,
            settings: { ...args.settings, ...settings },
          });
        }}
      />
    );
  },
  title: "Features/Editor/Island/Captions",
} satisfies Meta<typeof CaptionsIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// A template that sets one word apart: its size beside the type's.
export const KeyWord: Story = { args: { look: storyTextStyle("supersize") } };

// Hidden, there is nothing else to set, so the section folds away.
export const Hidden: Story = { args: { enabled: false } };

// Captions taken out of the video, to bring back.
export const CaptionsTakenOut: Story = { args: { hiddenCount: 2 } };

export const NoModel: Story = {
  args: {
    transcript: {
      model: { progress: 0, sizeBytes: MODEL_BYTES, status: "available" },
      status: "noModel",
    },
  },
};

export const Transcribing: Story = {
  args: { transcript: { progress: 0.4, status: "transcribing" } },
};

export const NothingSaid: Story = {
  args: { transcript: { status: "noSpeech" } },
};

export const Failed: Story = { args: { transcript: { status: "failed" } } };
