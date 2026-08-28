// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react";
import { Rabbit, Snail } from "lucide-react";

import { RadioGroup } from "../../../components/base/radio-group/radio-group";

import { IconRadio } from "./icon-radio";

const meta = {
  args: {
    icon: <Rabbit />,
    subtext: "Fast",
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
    <RadioGroup aria-label="Speed" defaultValue="fast" orientation="horizontal">
      <IconRadio icon={<Rabbit />} subtext="Fast" value="fast" />
      <IconRadio icon={<Snail />} subtext="Slow" value="slow" />
    </RadioGroup>
  ),
};
