// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react";
import { Rabbit, Snail } from "lucide-react";

import { RadioGroup } from "../../../components/base/radio-group/radio-group";

import { IconRadio } from "./icon-radio";

const meta = {
  args: {
    icon: <Rabbit />,
    value: "fast",
  },
  component: IconRadio,
  parameters: {
    controls: { disable: true },
    layout: "centered",
  },
  title: "Components/Icon Option",
} satisfies Meta<typeof IconRadio>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: () => (
    <RadioGroup
      aria-label="Speed"
      className="gap-tight"
      defaultValue="fast"
      orientation="horizontal"
    >
      <IconRadio aria-label="Fast" icon={<Rabbit />} value="fast" />
      <IconRadio aria-label="Slow" icon={<Snail />} value="slow" />
    </RadioGroup>
  ),
};
