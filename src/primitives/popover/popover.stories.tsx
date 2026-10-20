// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react-vite";
import { DialogTrigger } from "react-aria-components";

import { Button } from "../button/button";
import { Text } from "../text/text";

import { Popover } from "./popover";

const meta = {
  args: {
    "aria-label": "Details",
    children: <Text>A few choices that would crowd their row.</Text>,
  },
  component: Popover,
  parameters: { controls: { include: ["placement"] }, layout: "centered" },
  render: (args) => (
    <DialogTrigger defaultOpen>
      <Button>Open</Button>
      <Popover {...args} />
    </DialogTrigger>
  ),
  title: "Primitives/Popover",
} satisfies Meta<typeof Popover>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
