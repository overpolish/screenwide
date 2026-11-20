// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { EditorIslandStage } from "../editor-island-stage";
import {
  editorIslandStoryMeta,
  type EditorIslandStoryProps,
} from "../editor-island-story-meta";

import {
  soundIsland,
  type SoundIslandProps,
  type SoundTool,
  type SoundTrack,
} from "./sound-island";

import type { ToolPanelModel } from "../../tools/tool-panel-selection";
import type { Meta, StoryObj } from "@storybook/react-vite";

const AVAILABLE = (sizeBytes: number): ToolPanelModel => ({
  progress: 0,
  sizeBytes,
  status: "available",
});

const READY = (sizeBytes: number): ToolPanelModel => ({
  progress: 1,
  sizeBytes,
  status: "downloaded",
});

type Microphone = NonNullable<SoundTrack["microphone"]>;

const MICROPHONE: Microphone = {
  autoVolume: "on",
  ducking: { model: AVAILABLE(96_000_000), state: "off" },
  noise: "off",
  restore: "off",
  restoreModel: AVAILABLE(1_123_349_858),
  voice: "off",
};

const tracks = (microphone: Microphone, decibels = -4): SoundTrack[] => [
  { decibels, id: "microphone", label: "Microphone", microphone },
  { decibels: 0, id: "system", label: "System audio" },
];

// On lands at once, as a story has no track to clean.
const withTool = (
  microphone: Microphone,
  tool: SoundTool,
  isOn: boolean,
): Microphone => {
  const state = isOn ? "on" : "off";
  switch (tool) {
    case "autoVolume":
      return { ...microphone, autoVolume: state };
    case "reduceNoise":
      return { ...microphone, noise: state };
    case "restoreVoice":
      return { ...microphone, restore: state };
    case "smartDucking":
      return microphone.ducking
        ? { ...microphone, ducking: { ...microphone.ducking, state } }
        : microphone;
    case "vocalCleanup":
      return { ...microphone, voice: state };
  }
};

function SoundIsland({
  isShown,
  ...props
}: EditorIslandStoryProps & SoundIslandProps) {
  return <EditorIslandStage content={soundIsland(props)} isShown={isShown} />;
}

const meta = {
  ...editorIslandStoryMeta,
  args: {
    isShown: true,
    onDownloadRestoreVoice: fn(),
    onDownloadSmartDucking: fn(),
    onToolChange: fn(),
    onVolumeChange: fn(),
    tracks: tracks(MICROPHONE),
  },
  component: SoundIsland,
  decorators: [featureStage({ height: 360, width: 520 })],
  // Every choice changes the story's tracks, as it would the recording's. A
  // story cannot download a model, so a download lands at once.
  render: function Render(args) {
    const [, updateArgs] = useArgs<SoundIslandProps>();
    const changeMicrophone = (
      change: (microphone: Microphone) => Microphone,
    ) => {
      updateArgs({
        tracks: args.tracks.map((track) =>
          track.microphone
            ? { ...track, microphone: change(track.microphone) }
            : track,
        ),
      });
    };
    return (
      <SoundIsland
        {...args}
        onDownloadRestoreVoice={() => {
          args.onDownloadRestoreVoice();
          changeMicrophone((microphone) =>
            microphone.restoreModel
              ? {
                  ...microphone,
                  restoreModel: READY(microphone.restoreModel.sizeBytes),
                }
              : microphone,
          );
        }}
        onDownloadSmartDucking={() => {
          args.onDownloadSmartDucking();
          changeMicrophone((microphone) =>
            microphone.ducking?.model
              ? {
                  ...microphone,
                  ducking: {
                    ...microphone.ducking,
                    model: READY(microphone.ducking.model.sizeBytes),
                  },
                }
              : microphone,
          );
        }}
        onToolChange={(tool, isOn) => {
          args.onToolChange(tool, isOn);
          changeMicrophone((microphone) => withTool(microphone, tool, isOn));
        }}
        onVolumeChange={(trackId, decibels) => {
          args.onVolumeChange(trackId, decibels);
          updateArgs({
            tracks: args.tracks.map((track) =>
              track.id === trackId ? { ...track, decibels } : track,
            ),
          });
        }}
      />
    );
  },
  title: "Features/Editor/Island/Sound",
} satisfies Meta<typeof SoundIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Recording: Story = {};

export const RestoreVoice: Story = {
  args: {
    tracks: tracks({
      ...MICROPHONE,
      ducking: { model: READY(96_000_000), state: "on" },
      restore: "on",
      restoreModel: READY(1_123_349_858),
    }),
  },
};

export const DownloadingModel: Story = {
  args: {
    tracks: tracks({
      ...MICROPHONE,
      restoreModel: {
        progress: 0.45,
        sizeBytes: 1_123_349_858,
        status: "downloading",
      },
    }),
  },
};

// A recording of the system audio alone: no speech tools.
export const SystemAudioOnly: Story = {
  args: { tracks: [{ decibels: 0, id: "system", label: "System audio" }] },
};
