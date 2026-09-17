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
  keepAnnotationsBetweenSessions: false,
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
