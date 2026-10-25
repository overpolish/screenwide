// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { StoryGrid, StoryGridRow } from "../../../../../storybook/story-grid";

import { MeshPalette } from "./mesh-palette";

import type { Decorator, Meta, StoryObj } from "@storybook/react-vite";

// One panel at the width it is given in place; grids size themselves.
const panelWidth: Decorator = (Story) => (
  <div className="w-86">
    <Story />
  </div>
);

const colors = ["#1e1b4b", "#4f46e5", "#a855f7", "#38bdf8"];

const meta = {
  argTypes: {
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
  },
  args: {
    colors,
    isDisabled: false,
    locked: colors.map(() => false),
    onChange: fn(),
    onLockedChange: fn(),
  },
  component: MeshPalette,
  parameters: {
    controls: { include: ["isDisabled"] },
    layout: "centered",
  },
  // Picks, locks and new palettes land in the args, as the panel's document
  // would take them.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <MeshPalette
        {...args}
        onChange={(next) => {
          args.onChange(next);
          updateArgs({ colors: next });
        }}
        onLockedChange={(next) => {
          args.onLockedChange(next);
          updateArgs({ locked: next });
        }}
      />
    );
  },
  title: "Legacy/Editor/Tool Panels/Mesh Palette",
} satisfies Meta<typeof MeshPalette>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = { decorators: [panelWidth] };

// None locked, and two kept while the rest are made again.
function StateGrid({ isDisabled }: { isDisabled?: boolean }) {
  return (
    <StoryGrid>
      {[
        { label: "none locked", locked: [false, false, false, false] },
        { label: "two locked", locked: [true, false, true, false] },
      ].map(({ label, locked }) => (
        <StoryGridRow key={label} label={label}>
          <div className="w-86">
            <MeshPalette
              colors={colors}
              isDisabled={isDisabled}
              locked={locked}
              onChange={() => undefined}
              onLockedChange={() => undefined}
            />
          </div>
        </StoryGridRow>
      ))}
    </StoryGrid>
  );
}

export const States: Story = {
  parameters: { controls: { disable: true } },
  render: () => <StateGrid />,
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: () => <StateGrid isDisabled />,
};
