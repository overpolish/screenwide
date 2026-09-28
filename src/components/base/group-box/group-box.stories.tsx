// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react-vite";

import { ControlRow } from "../../shared/control-row/control-row";
import { Switch } from "../switch/switch";

import { GroupBox } from "./group-box";

const rows = (
  <>
    <ControlRow
      description="Show the containing folder after a successful export."
      title="Open location after export"
    >
      {(controlProps) => <Switch {...controlProps} defaultSelected />}
    </ControlRow>
    <ControlRow
      description="Play a sound when a recording starts and stops."
      title="Sounds"
    >
      {(controlProps) => <Switch {...controlProps} />}
    </ControlRow>
  </>
);

const meta = {
  args: { children: rows },
  component: GroupBox,
  decorators: [
    (Story) => (
      <div className="w-xl max-w-full">
        <Story />
      </div>
    ),
  ],
  parameters: { controls: { include: ["title"] }, layout: "padded" },
  title: "Primitives/Group Box",
} satisfies Meta<typeof GroupBox>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const WithTitle: Story = {
  args: { title: "Exporting" },
};
