// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Camera, Mic, Volume2 } from "lucide-react";
import { type ComponentProps, useState } from "react";
import { fn } from "storybook/test";

import { onIsland } from "../../storybook/on-island";

import { PillGroup, type PillGroupItem } from "./pill-group";

import type { Meta, StoryObj } from "@storybook/react-vite";

type PillGroupProps = ComponentProps<typeof PillGroup>;

// What each segment shows comes from its item, so one mixed group covers
// every way a segment renders: an icon alone, text alone, and both.
const items: PillGroupItem[] = [
  { ariaLabel: "System audio", icon: <Volume2 />, id: "system" },
  { id: "microphone", label: "Microphone" },
  { icon: <Camera />, id: "camera", label: "Camera" },
];

// Holds the selection, so pressing a segment moves the knob, and reports
// each change to the story's action.
function LiveGroup(props: PillGroupProps) {
  const [selected, setSelected] = useState(props.selected);
  if (props.selectionMode === "multiple") {
    const report = props.onSelectionChange;
    return (
      <PillGroup
        {...props}
        onSelectionChange={(next: string[]) => {
          report(next);
          setSelected(next);
        }}
        selected={selected as string[]}
      />
    );
  }
  const report = props.onSelectionChange;
  return (
    <PillGroup
      {...props}
      onSelectionChange={(next: string) => {
        report(next);
        setSelected(next);
      }}
      selected={selected as string}
    />
  );
}

const meta = {
  argTypes: {
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
  },
  args: {
    "aria-label": "Audio lanes",
    isDisabled: false,
    items,
    onSelectionChange: fn(),
    selected: "system",
  },
  component: PillGroup,
  parameters: {
    controls: { include: ["isDisabled"] },
    layout: "centered",
  },
  render: (args) => <LiveGroup {...args} />,
  title: "Primitives/Pill Group",
} satisfies Meta<typeof PillGroup>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Multiple: Story = {
  args: { selected: ["system", "camera"], selectionMode: "multiple" },
  parameters: { controls: { disable: true } },
};

export const DisabledSegment: Story = {
  args: { disabledIds: ["microphone"] },
  parameters: { controls: { disable: true } },
};

export const Disabled: Story = {
  args: { isDisabled: true },
  parameters: { controls: { disable: true } },
};

// On the island the segments are circles, so each item shows its icon, or a
// label short enough to fit one where it has none.
export const Island: Story = {
  args: {
    items: [
      { ariaLabel: "System audio", icon: <Volume2 />, id: "system" },
      { ariaLabel: "Microphone", icon: <Mic />, id: "microphone" },
      { ariaLabel: "Camera", icon: <Camera />, id: "camera" },
      { id: "all", label: "All" },
    ],
  },
  decorators: [onIsland()],
};
