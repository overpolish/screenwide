// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { PathField } from "./path-field";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    placeholder: { control: "text" },
    value: { control: "text" },
  },
  args: {
    "aria-label": "Recordings",
    isDisabled: false,
    onBrowse: fn(),
    value: "/Users/dom/Movies/Screenwide/Recordings",
  },
  component: PathField,
  parameters: {
    controls: {
      exclude: ["aria-label", "className", "onBrowse", "onReset", "resetLabel"],
    },
    layout: "centered",
  },
  title: "Composites/Path Field",
} satisfies Meta<typeof PathField>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

function StateGrid({
  isDisabled,
  onBrowse,
}: {
  onBrowse: () => void;
  isDisabled?: boolean;
}) {
  const shared = { "aria-label": "Recordings", isDisabled, onBrowse };
  return (
    <StoryGrid>
      <StoryGridRow label="chosen">
        <PathField
          {...shared}
          value="/Users/dom/Movies/Screenwide/Recordings"
        />
      </StoryGridRow>
      <StoryGridRow label="placeholder">
        <PathField {...shared} placeholder="Movies folder" value={null} />
      </StoryGridRow>
      <StoryGridRow label="long name">
        <PathField
          {...shared}
          value="/Users/dom/Movies/Client work for the spring launch"
        />
      </StoryGridRow>
      <StoryGridRow label="with reset">
        <PathField
          {...shared}
          onReset={fn()}
          resetLabel="Use the Movies folder"
          value="/Users/dom/Movies/Screenwide/Recordings"
        />
      </StoryGridRow>
    </StoryGrid>
  );
}

export const States: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onBrowse }) => <StateGrid onBrowse={onBrowse} />,
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onBrowse }) => <StateGrid isDisabled onBrowse={onBrowse} />,
};
