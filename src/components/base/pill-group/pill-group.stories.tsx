// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ComponentProps } from "react";

import { Camera, Mic, Volume2 } from "lucide-react";
import { useState } from "react";

import { PillGroup } from "./pill-group";

import type { Meta, StoryObj } from "@storybook/react-vite";

const items = [
  { icon: <Volume2 />, id: "system", label: "System audio" },
  { icon: <Mic />, id: "microphone", label: "Microphone" },
  { icon: <Camera />, id: "camera", label: "Camera" },
];

const meta = {
  component: PillGroup,
  parameters: { layout: "centered" },
  title: "Primitives/Pill Group",
} satisfies Meta<typeof PillGroup>;

export default meta;
type Story = StoryObj<typeof meta>;

function PillGroupExample(props: ComponentProps<typeof PillGroup>) {
  const [selected, setSelected] = useState(props.selected);
  return (
    <PillGroup {...props} onSelectionChange={setSelected} selected={selected} />
  );
}

export const Icons: Story = {
  args: {
    "aria-label": "Audio lanes",
    items,
    onSelectionChange: () => undefined,
    selected: "system",
  },
  render: (args) => <PillGroupExample {...args} />,
};

export const IconAndText: Story = {
  args: { ...Icons.args, display: "icon-label" },
  render: (args) => <PillGroupExample {...args} />,
};

export const Text: Story = {
  args: { ...Icons.args, display: "label" },
  render: (args) => <PillGroupExample {...args} />,
};

export const DisabledItem: Story = {
  args: {
    ...IconAndText.args,
    disabledIds: ["microphone"],
  },
  render: (args) => <PillGroupExample {...args} />,
};

export const Disabled: Story = {
  args: {
    ...IconAndText.args,
    isDisabled: true,
  },
  render: (args) => <PillGroupExample {...args} />,
};

export const CustomItemGeometry: Story = {
  args: {
    ...Icons.args,
    itemClassName: "size-12 rounded-xl [&_svg]:size-icon-prominent!",
  },
  render: (args) => <PillGroupExample {...args} />,
};
