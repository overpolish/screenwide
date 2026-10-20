// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react";
import { TooltipTrigger } from "react-aria-components";

import { Button } from "../button/button";

import { Tooltip } from "./tooltip";

const placements: React.ComponentProps<typeof Tooltip>["placement"][] = [
  "left",
  "top",
  "bottom",
  "end",
] as const;
const isolatedDocsStory = {
  docs: {
    story: {
      height: "160px",
      inline: false,
    },
  },
} as const;

/**
 * Must be using inside a `TooltipTrigger`.
 */
const meta = {
  argTypes: {
    children: { control: { disable: true } },
    className: { control: { disable: true } },
  },
  args: {
    children: "World!",
  },
  component: Tooltip,
  parameters: {
    controls: { disable: true },
    layout: "centered",
  },
  title: "Primitives/Tooltip",
} satisfies Meta<typeof Tooltip>;

export default meta;
type Story = StoryObj<typeof meta>;

/* --------------------------------- Stories -------------------------------- */
export const Default: Story = {
  render: (args) => (
    <TooltipTrigger>
      <Button>Hello</Button>
      <Tooltip {...args} placement="bottom" />
    </TooltipTrigger>
  ),
};

/** None exhaustive. */
export const Placements: Story = {
  parameters: {
    controls: { disable: true },
    ...isolatedDocsStory,
  },
  render: (args) => (
    <div className="flex items-center gap-section">
      {placements.map((placement) => (
        <TooltipTrigger isOpen key={placement}>
          <Button>Hello</Button>
          <Tooltip placement={placement} {...args} />
        </TooltipTrigger>
      ))}
    </div>
  ),
};
