// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { StoryGrid, StoryGridRow } from "../../../../storybook/story-grid";

import { ShortcutChip, type ShortcutChipProps } from "./shortcut-chip";

import type { Meta, StoryObj } from "@storybook/react-vite";

/** One shortcut on a row of the lane's height, its tick at the row's start
 * and room for its keys after it. */
function Chip(props: Omit<ShortcutChipProps, "x">) {
  return (
    <div className="relative h-5 w-20" dir="ltr">
      <ShortcutChip {...props} x={1} />
    </div>
  );
}

const SHORTCUTS = [
  { end: 1.5, id: "save", keys: ["Command", "S"], start: 0 },
  { end: 1.5, id: "shot", keys: ["Command", "Shift", "4"], start: 0 },
];

const meta = {
  args: {
    isSelected: false,
    onPress: fn(),
    shortcut: SHORTCUTS[0],
    x: 1,
  },
  component: ShortcutChip,
  parameters: { controls: { include: ["isSelected", "shortcut"] } },
  render: (args) => <Chip {...args} />,
  title: "Features/Editor/Timeline/Shortcut Chip",
} satisfies Meta<typeof ShortcutChip>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  decorators: [featureStage({ width: 80 })],
};

export const States: Story = {
  decorators: [featureStage({ width: 264 })],
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid columns={SHORTCUTS.map(({ keys }) => keys.join("+"))}>
      {[
        { isSelected: false, label: "Rest" },
        { isSelected: true, label: "Chosen" },
      ].map(({ label, ...state }) => (
        <StoryGridRow key={label} label={label}>
          {SHORTCUTS.map((shortcut) => (
            <Chip key={shortcut.id} {...args} {...state} shortcut={shortcut} />
          ))}
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};
