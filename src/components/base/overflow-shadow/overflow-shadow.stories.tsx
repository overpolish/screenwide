// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react-vite";

import { Button } from "../button/button";
import { ListBox } from "../listbox/listbox";
import { ListBoxItem } from "../listbox-item/listbox-item";

import { OverflowShadow } from "./overflow-shadow";

const items = Array.from(
  { length: 12 },
  (_, index) => `Item ${String(index + 1)}`,
);

const meta = {
  component: OverflowShadow,
  parameters: { layout: "centered" },
  title: "Primitives/Overflow Shadow",
} satisfies Meta<typeof OverflowShadow>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Vertical: Story = {
  render: () => (
    <OverflowShadow rootClassName="h-32 w-64 rounded-xl bg-content shadow-md">
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
    </OverflowShadow>
  ),
};

export const Horizontal: Story = {
  render: () => (
    <OverflowShadow
      className="gap-control p-control flex w-max"
      orientation="horizontal"
      rootClassName="h-9 w-64 rounded-xl bg-content shadow-md"
    >
      {items.map((item) => (
        <Button key={item} size="compact">
          {item}
        </Button>
      ))}
    </OverflowShadow>
  ),
};
