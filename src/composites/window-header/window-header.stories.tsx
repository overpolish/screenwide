// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ClipboardCopy, Upload } from "lucide-react";
import { useState } from "react";
import { fn } from "storybook/test";

import { Button } from "../../primitives/button/button";
import { featureStage } from "../../storybook/feature-story-stage";

import { WindowHeader } from "./window-header";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    platform: {
      control: "inline-radio",
      options: ["macos", "windows"],
    },
  },
  args: {
    onClose: fn(),
    onMinimize: fn(),
    onToggleMaximize: fn(),
    platform: "macos",
    title: "Settings",
  },
  component: WindowHeader,
  decorators: [featureStage({ height: 48, width: 720 })],
  parameters: {
    controls: {
      exclude: [
        "actions",
        "onClose",
        "onMinimize",
        "onTitleChange",
        "onToggleMaximize",
      ],
    },
    layout: "fullscreen",
  },
  title: "Composites/Window Header",
} satisfies Meta<typeof WindowHeader>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// A window toolbar's commands carry a leading icon; the one thing the window
// is for is primary.
export const WithActions: Story = {
  args: {
    actions: (
      <>
        <Button variant="ghost">
          <ClipboardCopy />
          Copy
        </Button>
        <Button color="primary">
          <Upload />
          Export
        </Button>
      </>
    ),
    title: "Screenwide 2026-08-08 at 14.32.05",
  },
};

export const EditableTitle: Story = {
  render: function Render(args) {
    const [title, setTitle] = useState("Untitled recording");
    return <WindowHeader {...args} onTitleChange={setTitle} title={title} />;
  },
};

export const Windows: Story = {
  args: { platform: "windows" },
};
