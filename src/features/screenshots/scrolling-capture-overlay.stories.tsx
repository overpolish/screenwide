// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Meta, type StoryObj } from "@storybook/react-vite";

import { FeatureStoryStage } from "../../storybook/feature-story-stage";

import { ScrollingCaptureOverlay } from "./scrolling-capture-overlay";

const meta = {
  args: { cancellable: true, phase: "working" },
  component: ScrollingCaptureOverlay,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage height={66} viewMode={context.viewMode} width={160}>
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
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
