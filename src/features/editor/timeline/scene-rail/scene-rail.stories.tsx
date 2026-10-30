// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { useRestingAction } from "../../../../storybook/use-resting-action";
import { recordedTime, timelineScale } from "../timeline-scale";

import { type TimelineScene } from "./scene-clip";
import { SceneRail, type SceneRailProps } from "./scene-rail";
import { storyScene } from "./scene-story-fixtures";

import type { Meta, StoryObj } from "@storybook/react-vite";

// A strip 120 seconds long, 640 px wide.
const WIDTH = 640;
const DURATION = 120;
const scale = timelineScale(recordedTime(DURATION));

/** The room between the neighbours of a moment, leaving out `except`. */
function roomAt(scenes: TimelineScene[], moment: number, except?: string) {
  const others = scenes.filter((other) => other.id !== except);
  return {
    from: Math.max(
      0,
      ...others.filter(({ end }) => end <= moment).map(({ end }) => end),
    ),
    to: Math.min(
      DURATION,
      ...others
        .filter(({ start }) => start >= moment)
        .map(({ start }) => start),
    ),
  };
}

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
          const start = Math.min(from, to);
          const end = Math.max(from, to);
          if (end - start < 1) return;
          const id = `scene-${String(scenes.length + 1)}`;
          const room = roomAt(scenes, from);
          const drawn = storyScene(id, "picture-in-picture", {
            end: Math.min(end, room.to),
            start: Math.max(start, room.from),
          });
          setScenes(
            [...scenes, drawn].sort(
              (first, second) => first.start - second.start,
            ),
          );
          setSelected(id);
        }}
        onRetime={(id, start, end) => {
          logRetime(id, start, end);
          const own = scenes.find((scene) => scene.id === id);
          if (!own) return;
          const room = roomAt(scenes, (own.start + own.end) / 2, id);
          setScenes(
            scenes.map((scene) =>
              scene.id === id
                ? {
                    ...scene,
                    end: Math.min(Math.max(end, start + 1), room.to),
                    start: Math.max(Math.min(start, end - 1), room.from),
                  }
                : scene,
            ),
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
