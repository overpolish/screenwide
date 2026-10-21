// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentProps } from "react";

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { Tooltip } from "./tooltip";

import type { Meta, StoryObj } from "@storybook/react-vite";

const contents: { label: string; props: ComponentProps<typeof Tooltip> }[] = [
  { label: "label", props: { label: "Reset the view" } },
  { label: "shortcut", props: { keys: ["C"], label: "Crop" } },
  {
    label: "modified shortcut",
    props: { keys: ["Command", "Z"], label: "Undo" },
  },
];

const meta = {
  args: { label: "Crop the recording" },
  component: Tooltip,
  parameters: {
    controls: { include: ["label"] },
    layout: "centered",
  },
  title: "Primitives/Tooltip",
} satisfies Meta<typeof Tooltip>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Contents: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <StoryGrid>
      {contents.map(({ label, props }) => (
        <StoryGridRow key={label} label={label}>
          <Tooltip {...props} />
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};

export const LongText: Story = {
  args: {
    label:
      "/Users/dom/Movies/Screenwide/Recordings/2026/September/Product walkthrough - final take.screenwide",
  },
  parameters: { controls: { disable: true } },
};
