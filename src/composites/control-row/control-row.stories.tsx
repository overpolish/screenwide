// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Disc } from "lucide-react";
import { useState } from "react";

import { Badge } from "../../primitives/badge/badge";
import { Switch } from "../../primitives/switch/switch";

import { ControlRow } from "./control-row";

import type { Meta, StoryObj } from "@storybook/react-vite";

// A switch as the row's control, live so the row can be tried.
function RowSwitch(controlProps: {
  "aria-labelledby": string;
  "aria-describedby"?: string;
}) {
  const [isOn, setIsOn] = useState(true);
  return <Switch {...controlProps} isSelected={isOn} onChange={setIsOn} />;
}

const meta = {
  argTypes: {
    description: { control: "text" },
    title: { control: "text" },
  },
  args: {
    children: (controlProps) => <RowSwitch {...controlProps} />,
    description: "Show it in Finder once the file is saved.",
    title: "Open folder after saving",
  },
  component: ControlRow,
  decorators: [
    (Story) => (
      <div className="w-80">
        <Story />
      </div>
    ),
  ],
  parameters: {
    controls: { include: ["description", "title"] },
    layout: "centered",
  },
  title: "Composites/Control Row",
} satisfies Meta<typeof ControlRow>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// An icon tile before the text, as the permissions window lists them.
export const WithLeading: Story = {
  args: {
    description: "Screen and system audio in recordings.",
    title: "Screen Recording",
  },
  render: (args) => (
    <ControlRow
      {...args}
      leading={
        <span
          aria-hidden="true"
          className="flex size-control-height items-center justify-center rounded-control bg-fill text-fg-secondary [&_svg]:size-icon"
        >
          <Disc />
        </span>
      }
    />
  ),
};

export const WithAccessory: Story = {
  args: { description: undefined, title: "Camera" },
  render: (args) => (
    <ControlRow {...args} titleAccessory={<Badge>Optional</Badge>} />
  ),
};
