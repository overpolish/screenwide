// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Meta, type StoryObj } from "@storybook/react-vite";

import { featureStage } from "../../storybook/feature-story-stage";

import { ScrollingCaptureOverlay } from "./scrolling-capture-overlay";

const meta = {
  args: { cancellable: true, phase: "working" },
  component: ScrollingCaptureOverlay,
  decorators: [featureStage({ height: 72, width: 160 })],
  parameters: { layout: "fullscreen" },
  title: "Legacy/Screenshots/Scrolling Capture Overlay",
} satisfies Meta<typeof ScrollingCaptureOverlay>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Working: Story = {};

export const Capturing: Story = { args: { phase: "capturing" } };

export const Stitching: Story = { args: { phase: "stitching" } };

// Another app can hold Escape, and then it is not offered.
export const NotCancellable: Story = { args: { cancellable: false } };

export const Finishing: Story = { args: { finished: true } };
