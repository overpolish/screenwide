// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Crop } from "lucide-react";

import { IconToggleButton } from "../../base/button/icon-button";

import { NativeTooltipSurface } from "./native-tooltip-surface";
import { NativeTooltipTrigger } from "./native-tooltip-trigger";

import type { Meta, StoryObj } from "@storybook/react-vite";

/**
 * The Editor's tooltips are drawn in a window of their own, because the
 * preview they stand over is a native surface the page cannot appear above.
 * There is no such window in a browser, so the trigger falls back to the
 * in-page tooltip here.
 */
const meta = {
  args: { content: { label: "Crop", shortcut: "C" } },
  component: NativeTooltipSurface,
  parameters: { layout: "centered" },
  title: "Components/Native Tooltip",
} satisfies Meta<typeof NativeTooltipSurface>;

export default meta;
type Story = StoryObj<typeof meta>;

/** The surface as the tooltip window draws it, at the size it asks for. */
export const Surface: Story = {};

/** Without a shortcut, a tooltip is the label alone. */
export const LabelOnly: Story = {
  args: { content: { label: "Reset the view" } },
};

/** The trigger, hovered or focused: in Storybook it shows the in-page
 * tooltip, and in the app the window. */
export const Trigger: Story = {
  render: (args) => (
    <NativeTooltipTrigger tooltip={args.content}>
      <IconToggleButton
        aria-keyshortcuts={args.content.shortcut}
        aria-label="Crop the recording"
        isSelected={false}
        onChange={() => undefined}
      >
        <Crop />
      </IconToggleButton>
    </NativeTooltipTrigger>
  ),
};
