// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../../../storybook/feature-story-stage";
import { toolPanelWidth } from "../../../popup-panel/layout";
import { firstAnnotationDress } from "../../annotations/annotation-defaults";
import { DEFAULT_CURSOR_EFFECTS } from "../../export/recording-export-settings";
import { ToolPanel } from "../tool-panel";
import { seedToolPanel } from "../tool-panel-story-seed";

import type { Meta, StoryObj } from "@storybook/react-vite";

/** The tool panel over an image: the picture file it shows, chosen or
 * replaced, and Flip beside it once placed; then its corners, its turn, its
 * shadow, whether a recording sways it, and whether it pops in over its
 * clip. An image has no colour of its own. */
const meta = {
  args: { tool: "annotation", workspace: "recording" },
  component: ToolPanel,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage viewMode={context.viewMode} width={toolPanelWidth}>
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Editor/Tool Panel/Image",
} satisfies Meta<typeof ToolPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

/** A chosen image, turned a little and casting a shadow. */
export const Chosen: Story = {
  beforeEach: () => {
    seedToolPanel({
      annotation: {
        angle: -Math.PI / 12,
        animated: true,
        id: "image-1",
        image: { aspect: 1, asset: "image:0123456789abcdef0123456789abcdef" },
        kind: "image",
        style: { ...firstAnnotationDress("image"), shadow: true },
      },
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      isLocked: false,
      selection: null,
    });
  },
};

/** A chosen image that sways over its clip, with the dice that gives it
 * another sway. */
export const Swaying: Story = {
  beforeEach: () => {
    seedToolPanel({
      annotation: {
        angle: 0,
        animated: true,
        id: "image-1",
        image: { aspect: 1, asset: "image:0123456789abcdef0123456789abcdef" },
        kind: "image",
        style: firstAnnotationDress("image"),
        sway: true,
      },
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      isLocked: false,
      selection: null,
    });
  },
};

/** The image tool before any picture has been chosen: Choose places the
 * first one. A fresh image always stands upright, so there is no turn to
 * choose yet and nothing to flip. */
export const Tool: Story = {
  beforeEach: () => {
    seedToolPanel({
      annotation: {
        animated: true,
        id: "",
        isDraft: true,
        kind: "image",
        style: firstAnnotationDress("image"),
      },
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      isLocked: false,
      selection: null,
    });
  },
};

/** A moving picture, one that loops, on a recording: the frame playback
 * starts on, and whether it loops or plays through once. */
export const Moving: Story = {
  beforeEach: () => {
    seedToolPanel({
      annotation: {
        animated: true,
        id: "image-1",
        image: {
          aspect: 1.4,
          asset: "image:0123456789abcdef0123456789abcdef",
          play: { cycleMs: 3_150, frame: 0, frames: 105, once: false },
        },
        kind: "image",
        style: firstAnnotationDress("image"),
      },
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      isLocked: false,
      selection: null,
    });
  },
};

/** The same picture on a screenshot, which keeps the one frame chosen. */
export const MovingOnAScreenshot: Story = {
  ...Moving,
  args: { tool: "annotation", workspace: "screenshot" },
};
