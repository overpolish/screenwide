// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AppWindowMac,
  AudioLines,
  Camera,
  Mic,
  Monitor,
  SquareDashed,
  Volume2,
} from "lucide-react";
import { type ComponentProps, useState } from "react";
import { fn } from "storybook/test";

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

/** The bezel-less form, at the regular size. The recording bar's ghost
 * control is the Capture story. */
export const Ghost: Story = {
  args: {
    ...Icons.args,
    display: "icon-label",
    variant: "ghost",
  },
  render: (args) => <PillGroupExample {...args} />,
};

const captureItems = [
  { icon: <Monitor />, id: "screen", label: "Screen" },
  { icon: <AppWindowMac />, id: "window", label: "Window" },
  { icon: <SquareDashed />, id: "region", label: "Region" },
  { icon: <Camera />, id: "camera", label: "Camera" },
  { icon: <AudioLines />, id: "audio", label: "Audio" },
];

/** The recording bar's control: icon-only 48 by 40 segments under 28px
 * glyphs. Each segment reports its presses after the selection has settled,
 * which is how a Screen segment opens its display chooser whether it was just
 * chosen or already was; the action log shows them. */
export const Capture: Story = {
  args: {
    "aria-label": "Recording type",
    items: captureItems.map((item) => ({
      ...item,
      onPress: fn().mockName(`onPress ${item.id}`),
    })),
    onSelectionChange: () => undefined,
    selected: "screen",
    size: "capture",
    variant: "ghost",
  },
  render: (args) => <PillGroupExample {...args} />,
};
