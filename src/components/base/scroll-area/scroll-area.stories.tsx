// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react-vite";

import { Button } from "../button/button";
import { ListBox } from "../listbox/listbox";
import { ListBoxItem } from "../listbox-item/listbox-item";

import { ScrollArea } from "./scroll-area";

const items = Array.from(
  { length: 12 },
  (_, index) => `Item ${String(index + 1)}`,
);

const meta = {
  args: { edgeEffect: "shadow", orientation: "vertical" },
  component: ScrollArea,
  parameters: { layout: "centered" },
  title: "Primitives/Scroll Area",
} satisfies Meta<typeof ScrollArea>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Vertical: Story = {
  render: (args) => (
    <ScrollArea {...args} rootClassName="h-32 w-64 rounded-xl bg-content">
      <ListBox
        aria-label="Overflowing items"
        className="w-full overflow-visible rounded-none bg-transparent shadow-none"
      >
        {items.map((item) => (
          <ListBoxItem id={item} key={item}>
            {item}
          </ListBoxItem>
        ))}
      </ListBox>
    </ScrollArea>
  ),
};

export const Horizontal: Story = {
  args: { orientation: "horizontal" },
  render: (args) => (
    <ScrollArea
      {...args}
      className="gap-control p-control flex w-max"
      rootClassName="h-9 w-64 rounded-xl bg-content"
    >
      {items.map((item) => (
        <Button key={item} size="compact">
          {item}
        </Button>
      ))}
    </ScrollArea>
  ),
};

export const NoEffect: Story = {
  ...Vertical,
  args: { edgeEffect: "none", scrollbarAutoHide: "never" },
};

export const Inset: Story = {
  ...Vertical,
  args: { edgeEffect: "inset", scrollbarAutoHide: "never" },
};
