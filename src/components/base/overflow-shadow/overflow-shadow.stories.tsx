// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react-vite";

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
    <OverflowShadow
      className="space-y-2 p-3"
      rootClassName="h-32 w-64 rounded-xl bg-neutral text-content-fg"
    >
      {items.map((item) => (
        <div className="rounded-lg bg-neutral px-3 py-2" key={item}>
          {item}
        </div>
      ))}
    </OverflowShadow>
  ),
};

export const Horizontal: Story = {
  render: () => (
    <OverflowShadow
      className="flex w-max gap-2 p-3"
      orientation="horizontal"
      rootClassName="h-16 w-64 rounded-xl bg-neutral text-content-fg"
    >
      {items.map((item) => (
        <div className="rounded-lg bg-neutral px-3 py-2" key={item}>
          {item}
        </div>
      ))}
    </OverflowShadow>
  ),
};
