// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../storybook/feature-story-stage";

import { GlidePreview } from "./glide-preview";
import { type GlideRegion } from "./glide-regions";

import type { Meta, StoryObj } from "@storybook/react-vite";

/** A full-height left cell, refined per story. */
const region = (
  cells: Partial<GlideRegion> & Pick<GlideRegion, "gridCols">,
): GlideRegion => ({
  colSpan: 1,
  colStart: 0,
  rowSpan: 2,
  rowStart: 0,
  ...cells,
});

/** Stands in for an extracted app icon, so Storybook needs no files. */
const sampleIcon =
  "data:image/svg+xml;utf8," +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">' +
      '<rect width="16" height="16" rx="4" fill="#4f7cff"/>' +
      '<circle cx="8" cy="8" r="3" fill="#fff"/></svg>',
  );

const meta = {
  args: {
    fit: null,
    iconSrc: null,
    pending: null,
    pulse: 0,
    region: region({ gridCols: 2 }),
  },
  component: GlidePreview,
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
  title: "Features/Glide Preview",
} satisfies Meta<typeof GlidePreview>;

export default meta;
type Story = StoryObj<typeof meta>;

export const LeftHalf: Story = {};

export const RightTwoThirds: Story = {
  args: { region: region({ colSpan: 2, colStart: 1, gridCols: 3 }) },
};

export const MiddleThird: Story = {
  args: { region: region({ colStart: 1, gridCols: 3 }) },
};

export const TopRightQuarter: Story = {
  args: { region: region({ colStart: 1, gridCols: 2, rowSpan: 1 }) },
};

export const BottomHalf: Story = {
  args: {
    region: region({ colSpan: 2, gridCols: 2, rowSpan: 1, rowStart: 1 }),
  },
};

export const Fill: Story = {
  args: { region: region({ colSpan: 2, gridCols: 2 }) },
};

/**
 * An app that refuses to widen past its own limit: the unmet part of the right
 * half stays neutral, while the extent it reached remains primary.
 */
export const ConstrainedRightHalf: Story = {
  args: {
    fit: {
      actual: { height: 1, width: 0.3, x: 0.7, y: 0 },
      fits: false,
    },
    region: region({ colStart: 1, gridCols: 2 }),
  },
};

/**
 * A window wider than the requested right half. Its overflow stays neutral;
 * only the area that overlaps the requested destination remains primary.
 */
export const ConstrainedRightHalfOverflow: Story = {
  args: {
    fit: {
      actual: { height: 1, width: 0.7, x: 0.3, y: 0 },
      fits: false,
    },
    region: region({ colStart: 1, gridCols: 2 }),
  },
};

export const Minimize: Story = {
  args: { pending: "minimize", region: null },
};

/** Re-armed from the bottom row: the hint wins, the row waits underneath. */
export const MinimizeOverBottomRow: Story = {
  args: {
    pending: "minimize",
    region: region({ colSpan: 2, gridCols: 2, rowSpan: 1, rowStart: 1 }),
  },
};

/** The glided app named in the middle of its own destination. */
export const WithAppIcon: Story = {
  args: { iconSrc: sampleIcon },
};
