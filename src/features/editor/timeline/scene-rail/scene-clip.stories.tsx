// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { StoryGrid, StoryGridRow } from "../../../../storybook/story-grid";
import { recordedTime, timelineScale } from "../timeline-scale";

import { SceneClip, type SceneClipProps } from "./scene-clip";
import { storyScene, turnedStoryScene } from "./scene-story-fixtures";

import type { Meta, StoryObj } from "@storybook/react-vite";

// A strip 32 seconds long, 320 px wide, so a second is 10 px.
const WIDTH = 320;
const scale = timelineScale(recordedTime(32));

function Clip(props: SceneClipProps) {
  return (
    <div className="relative h-5" dir="ltr" style={{ width: WIDTH }}>
      <SceneClip {...props} />
    </div>
  );
}

// A short scene in a grid leaves no room empty beside it.
function Fitted({
  seconds,
  ...props
}: Omit<SceneClipProps, "scale" | "width"> & { seconds: number }) {
  const width = seconds * 10;
  return (
    <div className="relative h-5" dir="ltr" style={{ width }}>
      <SceneClip
        {...props}
        scale={timelineScale(recordedTime(seconds))}
        scene={{ ...props.scene, end: seconds, start: 0 }}
        width={width}
      />
    </div>
  );
}

const meta = {
  args: {
    isSelected: false,
    onGrabBody: fn(),
    onGrabEdge: fn(),
    onPress: fn(),
    scale,
    scene: storyScene("scene", "split-two-thirds", { end: 26, start: 6 }),
    width: WIDTH,
  },
  component: SceneClip,
  parameters: { controls: { include: ["isSelected", "scene"] } },
  render: (args) => <Clip {...args} />,
  title: "Features/Editor/Timeline/Scene Clip",
} satisfies Meta<typeof SceneClip>;

export default meta;
type Story = StoryObj<typeof meta>;

// Each story's stage is as wide as what it draws, so it never scrolls.
export const Default: Story = {
  decorators: [featureStage({ width: WIDTH })],
};

// What a scene shows grows with its width; a scene a few pixels wide is
// still pressed across more than it is drawn.
export const Widths: Story = {
  decorators: [featureStage({ width: 336 })],
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid columns={["Rest", "Selected"]}>
      {[
        { label: "Bare", seconds: 2 },
        { label: "Layout", seconds: 4 },
        { label: "Cut short", seconds: 9 },
        { label: "Full", seconds: 13 },
      ].map(({ label, seconds }) => (
        <StoryGridRow key={label} label={label}>
          {[false, true].map((isSelected) => (
            <Fitted
              key={String(isSelected)}
              {...args}
              isSelected={isSelected}
              seconds={seconds}
            />
          ))}
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};

// Every layout as the Scene island's tiles draw it.
export const Layouts: Story = {
  decorators: [featureStage({ width: 720 })],
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid columns={["Rest", "Selected", "Turned"]}>
      {(
        [
          "split-two-thirds",
          "stacked",
          "picture-in-picture",
          "camera-only",
          "screen-only",
          "custom",
        ] as const
      ).map((layout) => {
        const scene = storyScene(layout, layout, { end: 18, start: 0 });
        const turned = turnedStoryScene(scene);
        return (
          <StoryGridRow key={layout} label={scene.name}>
            {[
              { isSelected: false, key: "rest", scene },
              { isSelected: true, key: "selected", scene },
              { isSelected: false, key: "turned", scene: turned },
            ].map((state) => (
              <Fitted
                key={state.key}
                {...args}
                isSelected={state.isSelected}
                scene={state.scene}
                seconds={18}
              />
            ))}
          </StoryGridRow>
        );
      })}
    </StoryGrid>
  ),
};
