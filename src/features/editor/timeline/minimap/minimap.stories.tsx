// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { useRestingAction } from "../../../../storybook/use-resting-action";
import { createPlayhead } from "../scrub-playhead";
import { takenOut } from "../strip/taken-out";
import { clampView, type View } from "../timeline-scale";
import {
  STORY_BEATS,
  STORY_DURATION,
  STORY_MAX_SQUEEZE,
  storySqueeze,
  STORY_TRACK_PX,
} from "../timeline-story-fixtures";

import { Minimap } from "./minimap";

import type { Meta, StoryObj } from "@storybook/react-vite";

// The fewest seconds the editor lets the strip zoom in to.
const FEWEST_S = 3;

// Three of the sample's beats are moments of different kinds.
const BEATS = STORY_BEATS.map((beat, index) => ({
  ...beat,
  color: ({ 2: "#ff8d28", 7: "#0088ff", 11: "#34c759" } as const)[index],
}));

type StoryProps = {
  onSqueezeChange: (squeeze: number) => void;
  onViewChange: (view: View) => void;
  // A share of the edited recording.
  playheadAt: number;
  squeeze: number;
  view: View;
};

// Keeps the window and squeeze as the strip does, logging a drag once it
// rests; Controls start it afresh.
function LiveMinimap({ playheadAt, ...props }: StoryProps) {
  const [playhead] = useState(createPlayhead);
  const [squeeze, setSqueeze] = useState(props.squeeze);
  const [view, setView] = useState(props.view);
  const logSqueeze = useRestingAction(props.onSqueezeChange);
  const logView = useRestingAction(props.onViewChange);
  const map = storySqueeze(squeeze);
  useEffect(() => {
    playhead.publish(playheadAt * map.duration, playheadAt);
  }, [map.duration, playhead, playheadAt]);
  return (
    // Room at the end for the squeeze's grip, which stands past the end.
    <div className="py-2 pe-5" dir="ltr" style={{ width: STORY_TRACK_PX }}>
      <Minimap
        beats={BEATS}
        map={map}
        maxSqueeze={STORY_MAX_SQUEEZE}
        onSqueezeChange={(next) => {
          logSqueeze(next);
          setSqueeze(next);
        }}
        onViewChange={(next) => {
          logView(next);
          setView(clampView(next, FEWEST_S / map.duration));
        }}
        playhead={playhead}
        squeeze={squeeze}
        taken={takenOut([], map.duration, STORY_DURATION)}
        view={view}
      />
    </div>
  );
}

const meta = {
  argTypes: {
    playheadAt: { control: { max: 1, min: 0, step: 0.01, type: "range" } },
    squeeze: {
      control: { max: STORY_MAX_SQUEEZE, min: 0, step: 1, type: "range" },
    },
  },
  args: {
    onSqueezeChange: fn(),
    onViewChange: fn(),
    playheadAt: 0.2,
    squeeze: 0,
    view: { end: 0.35, start: 0 },
  },
  component: LiveMinimap,
  decorators: [featureStage({ width: STORY_TRACK_PX })],
  parameters: {
    controls: { include: ["playheadAt", "squeeze", "view"] },
  },
  render: (args) => (
    <LiveMinimap key={JSON.stringify([args.squeeze, args.view])} {...args} />
  ),
  title: "Features/Editor/Timeline/Minimap",
} satisfies Meta<typeof LiveMinimap>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Picture: Story = {
  parameters: { controls: { include: ["playheadAt"] } },
  render: ({ playheadAt }) => (
    <div className="py-2" dir="ltr" style={{ width: STORY_TRACK_PX }}>
      <Minimap
        at={playheadAt}
        beats={BEATS}
        finds={[STORY_DURATION * 0.3, STORY_DURATION * 0.62]}
        isPicture
        map={storySqueeze(0)}
      />
    </div>
  ),
};
