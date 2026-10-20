// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Crop, MousePointer2, ScanSquare } from "lucide-react";
import { useState } from "react";

import logoUrl from "../../assets/screenwide-mark.svg";
import { IconToggleButton } from "../../primitives/button/icon-button";
import { Toolbar } from "../../primitives/toolbar/toolbar";
import { featureStage } from "../../storybook/feature-story-stage";

import { WindowHeader } from "./window-header";

import type { Meta, StoryObj } from "@storybook/react-vite";

const mark = (
  <img
    alt="Screenwide"
    className="brightness-0 dark:invert"
    draggable={false}
    src={logoUrl}
  />
);

const meta = {
  args: {
    onClose: () => undefined,
    title: "Open link",
  },
  component: WindowHeader,
  decorators: [
    (Story) => (
      <div className="window-surface text-content-fg">
        <Story />
      </div>
    ),
    featureStage({ height: 52, width: 672 }),
  ],
  parameters: { layout: "fullscreen" },
  title: "Composites/Window Header",
} satisfies Meta<typeof WindowHeader>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: { variant: "compact" },
};

export const Display: Story = {
  args: {
    leadingSection: mark,
    title: "Permissions",
    variant: "display",
  },
};

/** macOS: the OS draws the traffic lights over the inset the header leaves. */
export const MacOS: Story = {
  args: {
    leadingSection: mark,
    onMinimize: () => undefined,
    onToggleMaximize: () => undefined,
    platform: "macos",
    title: "Settings",
  },
};

/** Windows: the header draws the caption buttons and keeps no inset. */
export const Windows: Story = {
  args: {
    leadingSection: mark,
    onMinimize: () => undefined,
    onToggleMaximize: () => undefined,
    platform: "windows",
    title: "Settings",
  },
};

function EditableTitlePreview() {
  const [title, setTitle] = useState("Untitled recording");

  return (
    <WindowHeader
      leadingSection={mark}
      onClose={() => undefined}
      onTitleChange={setTitle}
      title={title}
    />
  );
}

export const EditableTitle: Story = {
  render: () => <EditableTitlePreview />,
};

export const LongTitle: Story = {
  args: {
    leadingSection: mark,
    title:
      "Screenwide product walkthrough - recording and screenshot editing - September 2026",
  },
};

/**
 * A unified toolbar: the tools take the window's centre and the title clips
 * against them rather than pushing them off it.
 */
export const WithCenter: Story = {
  args: {
    center: (
      <Toolbar aria-label="View tools">
        <IconToggleButton aria-label="Select" isSelected>
          <MousePointer2 />
        </IconToggleButton>
        <IconToggleButton aria-label="Resize canvas">
          <ScanSquare />
        </IconToggleButton>
        <IconToggleButton aria-label="Crop">
          <Crop />
        </IconToggleButton>
      </Toolbar>
    ),
    leadingSection: mark,
    title:
      "Screenwide product walkthrough - recording and screenshot editing - September 2026",
  },
};
