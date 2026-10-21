// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "../button/button";

import { ScrollArea } from "./scroll-area";

import type { Meta, StoryObj } from "@storybook/react-vite";

const items = Array.from(
  { length: 16 },
  (_, index) => `Recording ${String(index + 1)}`,
);

const meta = {
  argTypes: {
    edgeEffect: {
      control: "inline-radio",
      options: ["fade", "none"],
      table: { defaultValue: { summary: "fade" } },
    },
    scrollbarAutoHide: {
      control: "inline-radio",
      options: ["scroll", "never"],
      table: { defaultValue: { summary: "scroll" } },
    },
    scrollbarHidden: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
  },
  args: {
    edgeEffect: "fade",
    scrollbarAutoHide: "scroll",
    scrollbarHidden: false,
  },
  component: ScrollArea,
  parameters: {
    controls: {
      include: ["edgeEffect", "scrollbarAutoHide", "scrollbarHidden"],
    },
    layout: "centered",
  },
  // Bare, as a scroll area is: it has no surface of its own, so its bounds
  // show only as the fade and the scrollbar.
  render: (args) => (
    <ScrollArea {...args} className="flex flex-col" rootClassName="h-40 w-64">
      {items.map((item) => (
        <p className="py-1 text-body text-fg" key={item}>
          {item}
        </p>
      ))}
    </ScrollArea>
  ),
  title: "Primitives/Scroll Area",
} satisfies Meta<typeof ScrollArea>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Horizontal: Story = {
  args: { orientation: "horizontal" },
  render: (args) => (
    <ScrollArea
      {...args}
      className="flex w-max gap-separate"
      rootClassName="w-64"
    >
      {items.map((item) => (
        <Button key={item}>{item}</Button>
      ))}
    </ScrollArea>
  ),
};
