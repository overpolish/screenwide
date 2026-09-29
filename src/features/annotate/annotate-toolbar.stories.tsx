// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { FeatureStoryStage } from "../../storybook/feature-story-stage";
import { AnnotateSettings } from "../settings/types";

import { AnnotateToolbar, AnnotateToolbarProps } from "./annotate-toolbar";

import type { Meta, StoryObj } from "@storybook/react-vite";

const settings: AnnotateSettings = {
  defaultColor: "#ffcc00",
  defaultCounterAngle: 0,
  defaultCounterSize: 56,
  defaultHead: "end",
  defaultShape: "arrow",
  defaultWidth: 16,
  enabled: true,
  highlightHandDrawn: false,
  highlightManual: false,
  keepAnnotationsBetweenSessions: false,
  shapeHandDrawn: false,
  shapeRadius: 0,
  spotlightBlur: false,
  spotlightRadius: 12,
  spotlightSoftness: 10,
  toolbarPosition: null,
};

/** The choices are local to the story and the two actions report rather than
 * reaching the overlay, so nothing here draws or clears. */
function AnnotateToolbarExample(props: AnnotateToolbarProps) {
  const [chosen, setChosen] = useState(props.settings);
  return (
    <AnnotateToolbar
      {...props}
      onChange={(patch) => {
        setChosen((current) => ({ ...current, ...patch }));
      }}
      settings={chosen}
    />
  );
}

const meta = {
  args: {
    onChange: () => undefined,
    onClear: () => undefined,
    onDone: () => undefined,
    onUndo: () => undefined,
    savedColors: ["#2ec4b6", "#8b5e34"],
    settings,
  },
  component: AnnotateToolbar,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage viewMode={context.viewMode} width={620}>
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  render: (args) => <AnnotateToolbarExample {...args} />,
  title: "Features/Annotate Toolbar",
} satisfies Meta<typeof AnnotateToolbar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** The counter in hand: the disc sizes and the tail's aim stand where the
 * arrow's stroke and head were. */
export const Counter: Story = {
  args: { settings: { ...settings, defaultShape: "counter" } },
};

/** The highlight in hand: it fits the lines it covers, so the plate offers
 * how it is drawn in place of a size and a head. */
export const Highlight: Story = {
  args: { settings: { ...settings, defaultShape: "highlight" } },
};

/** A highlight laid by hand over a box, tinting what it covers. */
export const HighlightManual: Story = {
  args: {
    settings: { ...settings, defaultShape: "highlight", highlightManual: true },
  },
};

/** The shape in hand: its pen, whether it is drawn by hand, and how round
 * its corners are stand where the arrow's head was. */
export const Shape: Story = {
  args: {
    settings: {
      ...settings,
      defaultShape: "shape",
      shapeHandDrawn: true,
      shapeRadius: 20,
    },
  },
};

/** The spotlight in hand: no colour and no stroke, only how round its corners
 * are, how far its edge fades and whether what is outside it blurs. */
export const Spotlight: Story = {
  args: {
    settings: { ...settings, defaultShape: "spotlight", spotlightBlur: true },
  },
};
