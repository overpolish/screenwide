// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Meta, type StoryObj } from "@storybook/react-vite";

import { ScrollingCaptureOverlay } from "./scrolling-capture-overlay";

const meta = {
  args: { cancellable: true, phase: "working" },
  component: ScrollingCaptureOverlay,
  // The real window is 260x200 logical points and paints its own card.
  decorators: [
    (Story) => (
      <div className="h-[200px] w-[260px]">
        <Story />
      </div>
    ),
  ],
  parameters: { layout: "centered" },
  title: "Features/Scrolling Capture Overlay",
} satisfies Meta<typeof ScrollingCaptureOverlay>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Working: Story = {};

export const Capturing: Story = { args: { phase: "capturing" } };

export const Stitching: Story = { args: { phase: "stitching" } };

/** Escape can be taken by another app, and then it is not offered. */
export const NotCancellable: Story = { args: { cancellable: false } };

/** The moment between the capture settling and the window closing. */
export const Finishing: Story = { args: { finished: true } };
