// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../storybook/feature-story-stage";

import {
  GlideSpacePreview,
  type GlideSpacePreviewEvent,
} from "./glide-spaces-preview";

import type { Meta, StoryObj } from "@storybook/react-vite";

const sampleIcon =
  "data:image/svg+xml;utf8," +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">' +
      '<rect width="16" height="16" rx="4" fill="#4f7cff"/>' +
      '<circle cx="8" cy="8" r="3" fill="#fff"/></svg>',
  );

const event = (
  overrides: Partial<GlideSpacePreviewEvent> = {},
): GlideSpacePreviewEvent => ({
  count: 3,
  desktop: "writing",
  iconPath: sampleIcon,
  index: 1,
  origin: false,
  phase: "moving",
  selected: true,
  sessionId: 1,
  ...overrides,
});

const meta = {
  args: { event: event(), pulse: 0 },
  component: GlideSpacePreview,
  decorators: [
    (Story, context) =>
      context.parameters.productionStage === false ? (
        <Story />
      ) : (
        <FeatureStoryStage height={32} viewMode={context.viewMode} width={48}>
          <Story />
        </FeatureStoryStage>
      ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Glide Space Preview",
} satisfies Meta<typeof GlideSpacePreview>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Selected: Story = {};

export const Unselected: Story = {
  args: { event: event({ selected: false }) },
};

export const Ready: Story = {
  args: { event: event({ phase: "ready" }), pulse: 1 },
};

export const Origin: Story = {
  args: { event: event({ origin: true, selected: false }) },
};

export const SelectedOrigin: Story = {
  args: { event: event({ origin: true }) },
};
