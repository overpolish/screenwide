// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { featureStage } from "../../feature-story-stage";

import { EditorWindow } from "./editor-window";
import { GAME_SESSION } from "./recordings/game-session";
import { LOCAL_RECORDINGS } from "./recordings/local-recordings";
import { NATURAL_TAKE } from "./recordings/natural-take";
import { PODCAST } from "./recordings/podcast";
import {
  addingAnAnnotation,
  settingScenes,
  squeezingPauses,
  timingAnAnnotation,
} from "./tour/editor-tours";

import type { Meta, StoryObj } from "@storybook/react-vite";

// One working editor, to try at a top level: pick a tool in the left column
// and its options open on the island beside it; pick a pen and point at the
// screen; select an annotation and drag its bar or ends on the timeline;
// hover the strip, drag its end in, select a beat and press Delete. Press a
// pause (a spring) to choose it, double-click it to keep it, drag its end
// for a length of its own. Along the strip's top are the scenes: press one
// to change it, drag its ends to retime it, drag along the empty rail to
// add a zoom. Space plays; Escape steps back; the arrow keys move between
// beats and pauses. The stories named for a flow play it through by
// themselves, then leave the editor to carry on by hand.

const meta = {
  argTypes: {
    isVoiced: { control: "boolean" },
    kind: {
      control: "inline-radio",
      options: ["recording", "screenshot", "audio"],
      table: { defaultValue: { summary: "recording" } },
    },
  },
  component: EditorWindow,
  decorators: [featureStage({ window: "editor-recording" })],
  parameters: {
    controls: { include: ["isVoiced", "kind"] },
    layout: "fullscreen",
  },
  // Changing the kind or the voice starts the editor afresh.
  render: (args) => (
    <EditorWindow
      key={`${String(args.kind)}-${String(args.isVoiced)}`}
      {...args}
    />
  ),
  title: "Explorations/Editor",
} satisfies Meta<typeof EditorWindow>;

export default meta;
type Story = StoryObj<typeof meta>;

export const NoVoice: Story = { args: { isVoiced: false } };

export const WithVoice: Story = { args: { isVoiced: true } };

export const ThreeMinuteTake: Story = {
  args: { isVoiced: true, recording: NATURAL_TAKE, start: { playhead: 0 } },
};

export const GamingSession: Story = {
  args: { isVoiced: true, recording: GAME_SESSION, start: { playhead: 0 } },
};

export const Podcast: Story = {
  args: { isVoiced: true, recording: PODCAST, start: { playhead: 0 } },
};

const yours = LOCAL_RECORDINGS[0] as
  (typeof LOCAL_RECORDINGS)[number] | undefined;

export const YourRecording: Story = {
  args: {
    isVoiced: false,
    recording: yours?.recording,
    start: { playhead: 0 },
  },
  render: (args) =>
    args.recording ? (
      <EditorWindow key={yours?.name} {...args} />
    ) : (
      <p className="m-auto max-w-md text-body text-fg-secondary">
        No recording yet: add one to recordings/local/, as
        recordings/local-recordings.ts says.
      </p>
    ),
};

export const AnnotationSelected: Story = {
  args: {
    isVoiced: false,
    start: { playhead: 44, selection: { id: 3, kind: "mark" } },
  },
};

export const SceneSelected: Story = {
  args: {
    isVoiced: false,
    start: { playhead: 13, selection: { id: "scene-name", kind: "scene" } },
  },
};

export const AddingAnAnnotation: Story = {
  args: { isVoiced: false, start: { playhead: 20 } },
  play: ({ canvasElement }) => addingAnAnnotation(canvasElement),
};

export const TimingAnAnnotation: Story = {
  args: {
    isVoiced: false,
    start: { playhead: 44, selection: { id: 3, kind: "mark" } },
  },
  play: ({ canvasElement }) => timingAnAnnotation(canvasElement),
};

export const Scenes: Story = {
  args: { isVoiced: false, start: { playhead: 4 } },
  play: ({ canvasElement }) => settingScenes(canvasElement),
};

export const Screenshot: Story = { args: { kind: "screenshot" } };

export const SoundOnly: Story = {
  args: { kind: "audio", start: { playhead: 14 } },
};

export const SqueezingPauses: Story = {
  args: { isVoiced: false, start: { playhead: 20 } },
  play: ({ canvasElement }) => squeezingPauses(canvasElement),
};
