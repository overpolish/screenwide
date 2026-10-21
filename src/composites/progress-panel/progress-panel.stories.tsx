// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";
import { ConfirmActionButton } from "../confirm-action-button/confirm-action-button";

import { ProgressPanel } from "./progress-panel";

import type { Decorator, Meta, StoryObj } from "@storybook/react-vite";

// One panel at the width it is given in place; grids size themselves.
const panelWidth: Decorator = (Story) => (
  <div className="w-80">
    <Story />
  </div>
);

const meta = {
  argTypes: {
    label: { control: "text" },
    progress: { control: { max: 100, min: 0, type: "range" } },
    status: { control: "text" },
  },
  args: {
    label: "Saving recording",
    progress: 42,
    status: "Less than a minute remaining",
  },
  component: ProgressPanel,
  parameters: {
    controls: { exclude: ["actions", "progressLabel"] },
    layout: "centered",
  },
  title: "Composites/Progress Panel",
} satisfies Meta<typeof ProgressPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = { decorators: [panelWidth] };

export const States: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <StoryGrid>
      <StoryGridRow label="determinate">
        <div className="w-80">
          <ProgressPanel
            label="Downloading update"
            progress={64}
            status="64%"
          />
        </div>
      </StoryGridRow>
      <StoryGridRow label="indeterminate">
        <div className="w-80">
          <ProgressPanel label="Finishing recording" progress={null} />
        </div>
      </StoryGridRow>
    </StoryGrid>
  ),
};

export const WithActions: Story = {
  args: {
    actions: (
      <ConfirmActionButton
        confirmLabel="Confirm cancel"
        label="Cancel"
        onConfirm={fn()}
      />
    ),
  },
  decorators: [panelWidth],
};
