// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";
import { fn } from "storybook/test";

import { ISLAND } from "../../../primitives/island/island-geometry";
import { featureStage } from "../../../storybook/feature-story-stage";
import { PREVIEW_NOTCH } from "../../../storybook/preview-notch";
import { StoryNotch } from "../../../storybook/story-notch";

import { AnnotateIsland, type AnnotateIslandProps } from "./annotate-island";

import type { AnnotateSettings } from "../../../bindings/AnnotateSettings";
import type { Meta, StoryObj } from "@storybook/react-vite";

const settings: AnnotateSettings = {
  defaultColor: "#ffcc00",
  defaultCounterAngle: 0,
  defaultCounterSize: 36,
  defaultHead: "end",
  defaultShape: "arrow",
  defaultWidth: 12,
  enabled: true,
  highlightHandDrawn: false,
  highlightManual: false,
  highlightTint: false,
  keepAnnotationsBetweenSessions: false,
  shapeHandDrawn: false,
  shapeRadius: 0,
  spotlightBlur: true,
  spotlightRadius: 25,
  spotlightSoftness: 10,
  toolbarPosition: null,
};

/** The island at the top of the canvas, under the notch when there is one.
 * Choices are kept by the story, so every one can be tried. */
function AnnotateIslandStage(props: AnnotateIslandProps) {
  const [chosen, setChosen] = useState(props.settings);
  return (
    <div className="relative size-full" dir="ltr">
      <div
        className="absolute inset-x-0"
        style={{ top: props.notch ? 0 : ISLAND.screenGap }}
      >
        <AnnotateIsland
          {...props}
          onChange={(patch, immediate) => {
            props.onChange(patch, immediate);
            setChosen((current) => ({ ...current, ...patch }));
          }}
          settings={chosen}
        />
      </div>
      {props.notch ? <StoryNotch notch={props.notch} /> : null}
    </div>
  );
}

const meta = {
  argTypes: {
    isShown: { table: { defaultValue: { summary: "true" } } },
  },
  args: {
    isShown: true,
    onChange: fn(),
    onClear: fn(),
    onDone: fn(),
    onUndo: fn(),
    savedColors: ["#2ec4b6", "#8b5e34"],
    settings,
  },
  component: AnnotateIsland,
  decorators: [
    (Story, { args, globals }) =>
      Story({
        args: {
          ...args,
          notch: globals.notch === "off" ? null : PREVIEW_NOTCH,
        },
      }),
    featureStage({ height: 320, width: 720 }),
  ],
  parameters: {
    controls: { include: ["isShown"] },
    layout: "fullscreen",
    notch: true,
  },
  render: (args) => <AnnotateIslandStage {...args} />,
  title: "Features/Annotate/Island",
} satisfies Meta<typeof AnnotateIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Arrow: Story = {};

export const Counter: Story = {
  args: { settings: { ...settings, defaultShape: "counter" } },
};

export const Highlight: Story = {
  args: { settings: { ...settings, defaultShape: "highlight" } },
};

export const Shape: Story = {
  args: { settings: { ...settings, defaultShape: "shape" } },
};

export const Focus: Story = {
  args: { settings: { ...settings, defaultShape: "spotlight" } },
};

export const Draw: Story = {
  args: { settings: { ...settings, defaultShape: "draw" } },
};

export const Colours: Story = { args: { initialSection: "colors" } };

/** `count` colours of your own around the colour wheel, in two lightnesses,
 * as `#rrggbb`. */
const savedAround = (count: number) =>
  Array.from({ length: count }, (_, index) => {
    const hue = (index * 360) / count;
    const lightness = index % 2 === 0 ? 0.55 : 0.4;
    const channel = (shift: number) => {
      const turn = (shift + hue / 30) % 12;
      const amount =
        lightness -
        0.6 *
          Math.min(lightness, 1 - lightness) *
          Math.max(-1, Math.min(turn - 3, 9 - turn, 1));
      return Math.round(amount * 255)
        .toString(16)
        .padStart(2, "0");
    };
    return `#${channel(0)}${channel(8)}${channel(4)}`;
  });

export const FiftyFourColours: Story = {
  args: { initialSection: "colors", savedColors: savedAround(46) },
};
