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

import type { ToolPanelModel } from "../../tool-panels/tool-panel-selection";
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
  autoVolumeProgress: 0,
  ducking: { model: AVAILABLE(96_000_000), progress: 0, state: "off" },
  noise: "off",
  noiseProgress: 0,
  studio: "off",
  studioModel: AVAILABLE(310_000_000),
  studioProgress: 0,
  voice: "off",
  voiceProgress: 0,
};

const tracks = (microphone: Microphone, decibels = -4): SoundTrack[] => [
  { decibels, id: "microphone", label: "Microphone", microphone },
  { decibels: 0, id: "system", label: "System audio" },
];

/** A tool turned on or off in a story: on lands at once, as a story has no
 * track to clean. */
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
    case "studioDucking":
      return microphone.ducking
        ? { ...microphone, ducking: { ...microphone.ducking, state } }
        : microphone;
    case "studioSound":
      return { ...microphone, studio: state };
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
    onDownloadStudioDucking: fn(),
    onDownloadStudioSound: fn(),
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
        onDownloadStudioDucking={() => {
          args.onDownloadStudioDucking();
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
        onDownloadStudioSound={() => {
          args.onDownloadStudioSound();
          changeMicrophone((microphone) =>
            microphone.studioModel
              ? {
                  ...microphone,
                  studioModel: READY(microphone.studioModel.sizeBytes),
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

export const StudioSound: Story = {
  args: {
    tracks: tracks({
      ...MICROPHONE,
      ducking: { model: READY(96_000_000), progress: 0, state: "on" },
      studio: "on",
      studioModel: READY(310_000_000),
    }),
  },
};

// Reduce noise and Auto volume both at work, the first time each is asked
// for.
export const Working: Story = {
  args: {
    tracks: tracks({
      ...MICROPHONE,
      autoVolume: "cleaning",
      autoVolumeProgress: 0.6,
      noise: "cleaning",
      noiseProgress: 0.3,
    }),
  },
};

export const DownloadingModel: Story = {
  args: {
    tracks: tracks({
      ...MICROPHONE,
      studioModel: {
        progress: 0.45,
        sizeBytes: 310_000_000,
        status: "downloading",
      },
    }),
  },
};

// A recording of the system audio alone: no speech tools.
export const SystemAudioOnly: Story = {
  args: { tracks: [{ decibels: 0, id: "system", label: "System audio" }] },
};
