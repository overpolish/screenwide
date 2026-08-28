// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react";
import { WandSparkles } from "lucide-react";

import { Button } from "../../base/button/button";
import { IconButton } from "../../base/button/icon-button";

import { CheckOnClick } from "./check-on-click";

const meta = {
  component: CheckOnClick,
  parameters: {
    layout: "centered",
  },
  title: "Components/Check on Click",
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
      <IconButton aria-label="Apply" size="compact">
        <WandSparkles size={14} />
      </IconButton>
    ),
    onPress: () =>
      new Promise((resolve) => {
        setTimeout(resolve, 1500);
      }),
  },
};
