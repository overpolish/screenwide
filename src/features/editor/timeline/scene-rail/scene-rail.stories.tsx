// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { useRestingAction } from "../../../../storybook/use-resting-action";
import { recordedTime, timelineScale } from "../timeline-scale";

import { SceneRail, type SceneRailProps } from "./scene-rail";
import { keptScene, sceneDrawn, storyScene } from "./scene-story-fixtures";

import type { Meta, StoryObj } from "@storybook/react-vite";

// A strip 120 seconds long, 640 px wide.
const WIDTH = 640;
const DURATION = 120;
const scale = timelineScale(recordedTime(DURATION));

/** The rail, keeping what is chosen, retimed and drawn, each scene kept
 * between its neighbours, as the editor keeps them; Controls start it
 * afresh with what they set. */
function LiveRail(args: SceneRailProps) {
  const [scenes, setScenes] = useState(args.scenes);
  const [selected, setSelected] = useState(args.selected);
  const logRetime = useRestingAction(args.onRetime);
  return (
    <div className="py-2" style={{ width: WIDTH }}>
      <SceneRail
        {...args}
        onDraw={(from, to) => {
          args.onDraw(from, to);
          const drawn = sceneDrawn({ duration: DURATION, scenes }, from, to);
          if (!drawn) return;
          setScenes(
            [...scenes, drawn].sort(
              (first, second) => first.start - second.start,
            ),
          );
          setSelected(drawn.id);
        }}
        onRetime={(id, start, end) => {
          logRetime(id, start, end);
          setScenes(
            keptScene({ duration: DURATION, scenes }, id, { end, start }),
          );
        }}
        onSelect={(id) => {
          args.onSelect(id);
          setSelected(id);
        }}
        scenes={scenes}
        selected={selected}
      />
    </div>
  );
}

const meta = {
  args: {
    duration: DURATION,
    onDraw: fn(),
    onRetime: fn(),
    onSelect: fn(),
    scale,
    scenes: [
      storyScene("intro", "camera-only", { end: 8, start: 0 }),
      storyScene("demo", "split-two-thirds", { end: 40, start: 12 }),
      storyScene("detail", "picture-in-picture", { end: 70, start: 58 }),
      storyScene("brief", "screen-only", { end: 86, start: 84 }),
    ],
    width: WIDTH,
  },
  component: SceneRail,
  decorators: [featureStage({ width: WIDTH })],
  parameters: { controls: { include: ["scenes", "selected"] } },
  render: (args) => (
    <LiveRail key={JSON.stringify([args.scenes, args.selected])} {...args} />
  ),
  title: "Features/Editor/Timeline/Scene Rail",
} satisfies Meta<typeof SceneRail>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// Nothing framed yet: hovering the rail shows where a drag draws a scene.
export const Empty: Story = { args: { scenes: [] } };
