// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../../../storybook/feature-story-stage";
import { toolPanelWidth } from "../../../popup-panel/layout";
import { firstAnnotationDress } from "../../annotations/annotation-defaults";
import { DEFAULT_CURSOR_EFFECTS } from "../../export/recording-export-settings";
import { DEFAULT_STICKER } from "../../stickers/sticker-library";
import { ToolPanel } from "../tool-panel";
import { seedToolPanel } from "../tool-panel-story-seed";

import type { Meta, StoryObj } from "@storybook/react-vite";

/** The tool panel over a sticker: the emoji it shows, picked as the system
 * draws it, or a picture file chosen with Image; then its corners, its turn,
 * its shadow, whether it pops in over its clip, and Flip. A sticker has no
 * colour of its own. */
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
  title: "Features/Editor/Tool Panel/Sticker",
} satisfies Meta<typeof ToolPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

/** A chosen sticker, turned a little and casting a shadow. */
export const Chosen: Story = {
  beforeEach: () => {
    seedToolPanel({
      annotation: {
        angle: -Math.PI / 12,
        animated: true,
        id: "sticker-1",
        kind: "sticker",
        sticker: { aspect: 1, asset: "emoji:🔥" },
        style: { ...firstAnnotationDress("sticker"), shadow: true },
      },
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      isLocked: false,
      selection: null,
    });
  },
};

/** The sticker tool with nothing chosen: the picture the next sticker shows,
 * which always stands upright, so there is no turn to choose yet and nothing
 * to flip. */
export const Tool: Story = {
  beforeEach: () => {
    seedToolPanel({
      annotation: {
        animated: true,
        id: "",
        isDraft: true,
        kind: "sticker",
        sticker: DEFAULT_STICKER,
        style: firstAnnotationDress("sticker"),
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
        id: "sticker-1",
        kind: "sticker",
        sticker: {
          aspect: 1.4,
          asset: "image:0123456789abcdef0123456789abcdef",
          play: { cycleMs: 3_150, frame: 0, frames: 105, once: false },
        },
        style: firstAnnotationDress("sticker"),
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
