// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../../storybook/feature-story-stage";
import { toolPanelWidth } from "../../popup-panel/layout";
import { firstAnnotationDress } from "../annotation-defaults";
import { DEFAULT_CURSOR_EFFECTS } from "../recording-export-settings";

import { ToolPanel } from "./tool-panel";
import { seedToolPanel } from "./tool-panel-story-seed";

import type { Meta, StoryObj } from "@storybook/react-vite";

/** The tool panel of a drawing tool with nothing chosen: the dress the tool
 * draws its next annotation in, picked before drawing. */
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
  title: "Features/Editor/Tool Panel/Drawing Tool",
} satisfies Meta<typeof ToolPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

/** The arrow tool: its size, head and colour, with nothing to turn round
 * until an arrow is drawn. */
export const Arrow: Story = {
  beforeEach: () => {
    seedToolPanel({
      annotation: {
        animated: true,
        id: "",
        isDraft: true,
        kind: "arrow",
        style: firstAnnotationDress("arrow"),
      },
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      isLocked: false,
      selection: null,
    });
  },
};

/** The highlight tool: whether the next one fits the text or is laid by hand
 * is chosen before it is drawn. */
export const Highlight: Story = {
  beforeEach: () => {
    seedToolPanel({
      annotation: {
        animated: true,
        id: "",
        isDraft: true,
        kind: "highlight",
        style: { ...firstAnnotationDress("highlight"), handDrawn: true },
      },
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      isLocked: false,
      selection: null,
    });
  },
};

/** The shape tool: one outline whose corners the radius rounds, drawn clean
 * or by hand, chosen before it is drawn. */
export const Shape: Story = {
  beforeEach: () => {
    seedToolPanel({
      annotation: {
        animated: true,
        id: "",
        isDraft: true,
        kind: "shape",
        style: {
          ...firstAnnotationDress("shape"),
          handDrawn: true,
          radius: 20,
        },
      },
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      isLocked: false,
      selection: null,
    });
  },
};

/** The spotlight tool: a rounded box the picture dims around, with its edge's
 * fade and whether what is outside it blurs, chosen before it is drawn. */
export const Spotlight: Story = {
  beforeEach: () => {
    seedToolPanel({
      annotation: {
        animated: true,
        id: "",
        isDraft: true,
        kind: "spotlight",
        style: { ...firstAnnotationDress("spotlight"), blur: true },
      },
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      isLocked: false,
      selection: null,
    });
  },
};
