// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Crop } from "lucide-react";
import { ComponentProps } from "react";

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";
import { IconButton } from "../button/icon-button";

import { Tooltip } from "./tooltip";
import { TooltipTrigger } from "./tooltip-trigger";

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

// The tooltip window only exists in the app: hovering shows it in
// `pnpm dev:storybook-native`, and nothing in a browser.
export const Trigger: Story = {
  render: ({ label }) => (
    <TooltipTrigger tooltip={label}>
      <IconButton aria-label="Crop" variant="ghost">
        <Crop />
      </IconButton>
    </TooltipTrigger>
  ),
};
