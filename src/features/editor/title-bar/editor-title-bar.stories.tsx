// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ComponentProps } from "react";

import { useState } from "react";
import { fn } from "storybook/test";

import { featureStage } from "../../../storybook/feature-story-stage";
import { StoryGrid, StoryGridRow } from "../../../storybook/story-grid";
import { recording, screenshot } from "../panel/editor-story-fixtures";
import { useRecordingFrameCopy } from "../recording/use-copy-recording-frame";
import {
  type CaptureParts,
  type EditorKind,
  type EditorMode,
  NO_CAPTURE_PARTS,
} from "../toolbar/editor-modes";

import { EditorModeBar } from "./editor-mode-bar";
import { EditorTitleBar } from "./editor-title-bar";

import type { Meta, StoryObj } from "@storybook/react-vite";

const EVERY_PART: CaptureParts = {
  audio: true,
  camera: true,
  cursor: true,
  keyboard: true,
  microphone: true,
  picture: "screen",
  transcript: "heard",
};

const AUDIO_PARTS: CaptureParts = {
  ...EVERY_PART,
  camera: false,
  cursor: false,
  keyboard: false,
  picture: null,
};

// Pressed open and shut as in the editor.
function Modes({
  isDisabled,
  kind,
  onModeChange,
  parts = kind === "screenshot" ? NO_CAPTURE_PARTS : EVERY_PART,
}: {
  kind: EditorKind;
  onModeChange: (mode: EditorMode | null) => void;
  isDisabled?: boolean;
  // Every part by default.
  parts?: CaptureParts;
}) {
  const [mode, setMode] = useState<EditorMode | null>(null);
  const [isTracksOpen, setIsTracksOpen] = useState(false);
  return (
    <EditorModeBar
      isDisabled={isDisabled}
      kind={kind}
      mode={mode}
      onModeChange={(next) => {
        onModeChange(next);
        setMode(next);
      }}
      parts={parts}
      tracks={
        kind === "screenshot"
          ? null
          : { isOpen: isTracksOpen, onOpenChange: setIsTracksOpen }
      }
    />
  );
}

const onModeChange = fn();

// What the recording holds in its export is in Controls, so the modes are
// seen growing in and shrinking out as each is switched.
type StoryArgs = ComponentProps<typeof EditorTitleBar> & CaptureParts;

const meta = {
  argTypes: {
    isSaving: { table: { defaultValue: { summary: "false" } } },
    picture: { control: "inline-radio", options: ["screen", "camera", null] },
    transcript: {
      control: "inline-radio",
      options: ["pending", "heard", "silent"],
    },
  },
  args: {
    ...EVERY_PART,
    artifact: recording,
    canExport: true,
    isSaving: false,
    onClose: fn(),
    onCopy: fn(),
    onDelete: fn(),
    onExport: fn(),
    onMinimize: fn(),
    onRename: fn(),
    onToggleMaximize: fn(),
  },
  // A wide editor.
  decorators: [featureStage({ width: 1600 })],
  // A recording's Copy Frame, which its preview offers in the editor.
  loaders: [
    () => {
      useRecordingFrameCopy.setState({ copy: fn(() => Promise.resolve()) });
    },
  ],
  parameters: {
    controls: {
      include: [
        "canExport",
        "isSaving",
        "picture",
        "camera",
        "cursor",
        "keyboard",
        "microphone",
        "transcript",
        "audio",
      ],
    },
  },
  render: ({
    audio,
    camera,
    cursor,
    keyboard,
    microphone,
    picture,
    transcript,
    ...args
  }) => (
    <EditorTitleBar
      {...args}
      modes={
        <Modes
          isDisabled={args.isSaving}
          kind="recording"
          onModeChange={onModeChange}
          parts={{
            audio,
            camera,
            cursor,
            keyboard,
            microphone,
            picture,
            transcript,
          }}
        />
      }
    />
  ),
  title: "Features/Editor/Title Bar",
} satisfies Meta<StoryArgs>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// The editor window's minimum width.
const MINIMUM_WIDTH = 800;

export const MinimumWidth: Story = {
  decorators: [featureStage({ width: MINIMUM_WIDTH + 160 })],
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid>
      <StoryGridRow label="recording">
        <div style={{ width: MINIMUM_WIDTH }}>
          <EditorTitleBar
            {...args}
            artifact={recording}
            modes={<Modes kind="recording" onModeChange={onModeChange} />}
          />
        </div>
      </StoryGridRow>
      <StoryGridRow label="screenshot">
        <div style={{ width: MINIMUM_WIDTH }}>
          <EditorTitleBar
            {...args}
            artifact={screenshot}
            modes={<Modes kind="screenshot" onModeChange={onModeChange} />}
          />
        </div>
      </StoryGridRow>
      <StoryGridRow label="audio">
        <div style={{ width: MINIMUM_WIDTH }}>
          <EditorTitleBar
            {...args}
            artifact={recording}
            modes={
              <Modes
                kind="audio"
                onModeChange={onModeChange}
                parts={AUDIO_PARTS}
              />
            }
          />
        </div>
      </StoryGridRow>
    </StoryGrid>
  ),
};
