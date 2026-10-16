// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../../../storybook/feature-story-stage";

import { ExportPreviewCompare } from "./export-preview-compare";
import { storyFrames } from "./export-preview-story-fixtures";

import type { Meta, StoryObj } from "@storybook/react-vite";

const frames = storyFrames();

const meta = {
  args: {
    className: "h-full w-full",
    exported: frames.exported,
    failed: false,
    isUpdating: false,
    original: frames.original,
  },
  component: ExportPreviewCompare,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage height={470} viewMode={context.viewMode} width={416}>
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Editor/Export Preview",
} satisfies Meta<typeof ExportPreviewCompare>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Half size at the smallest file, against Original. */
export const Comparing: Story = {};

/** Newer settings are being made; the frames shown are of the last ones. */
export const Updating: Story = {
  args: { isUpdating: true },
};

/** The first frame is still being made. */
export const Making: Story = {
  args: { exported: null, isUpdating: true, original: null },
};

export const Failed: Story = {
  args: { exported: null, failed: true, original: null },
};
