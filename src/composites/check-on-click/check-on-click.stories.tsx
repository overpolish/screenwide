// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react";
import { WandSparkles } from "lucide-react";

import { Button } from "../../primitives/button/button";
import { IconButton } from "../../primitives/button/icon-button";

import { CheckOnClick } from "./check-on-click";

const meta = {
  component: CheckOnClick,
  parameters: {
    layout: "centered",
  },
  title: "Composites/Check on Click",
} satisfies Meta<typeof CheckOnClick>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ButtonExample: Story = {
  args: {
    children: <Button>Action</Button>,
    onPress: () => undefined,
  },
};

export const IconButtonExample: Story = {
  args: {
    children: (
      <IconButton aria-label="Apply">
        <WandSparkles />
      </IconButton>
    ),
    onPress: () =>
      new Promise((resolve) => {
        setTimeout(resolve, 1500);
      }),
  },
};
