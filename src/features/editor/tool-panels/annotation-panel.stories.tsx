// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../../storybook/feature-story-stage";
import { toolPanelWidth } from "../../popup-panel/layout";
import { DEFAULT_CURSOR_EFFECTS } from "../recording-export-settings";

import { ToolPanel } from "./tool-panel";
import { seedToolPanel } from "./tool-panel-story-seed";

import type { Meta, StoryObj } from "@storybook/react-vite";

const seed = seedToolPanel;

/** The dress fields a story's annotation leaves at their defaults. */
const dress = {
  align: "left",
  head: "none",
  radius: 0,
  redaction: "erase",
  strength: 0,
} as const;

/** The tool panel over a chosen annotation: its controls follow the kind of
 * annotation in hand rather than a tool. */
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
  title: "Features/Editor/Tool Panel/Annotation",
} satisfies Meta<typeof ToolPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

/** The annotation panel over a chosen counter: a disc with a number in it has
 * no head to choose and no ends to swap, so the panel offers its size, whether
 * it arrives over its clip, and its colour. */
export const Counter: Story = {
  args: { tool: "annotation", workspace: "recording" },
  beforeEach: () => {
    seed({
      annotation: {
        // A tail turned a quarter past east, so the Angle row shows an aim
        // rather than its own default.
        angle: Math.PI / 4,
        animated: true,
        id: "counter-1",
        kind: "counter",
        style: { ...dress, color: "#ffcc00", width: 56 },
      },
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      isLocked: false,
      selection: null,
    });
  },
};

/** The Text panel: a text box has no head or aim, and lines its lines up by
 * the alignment it carries instead. */
export const Text: Story = {
  args: { tool: "annotation", workspace: "screenshot" },
  beforeEach: () => {
    seed(
      {
        annotation: {
          animated: true,
          id: "text-1",
          kind: "text",
          style: { ...dress, align: "center", color: "#ffcc00", width: 28 },
        },
        cursorEffects: DEFAULT_CURSOR_EFFECTS,
        frame: null,
        isLocked: false,
        selection: null,
      },
      "screenshot",
    );
  },
};

/** The Redaction panel, pixelating securely: a style and a block size are
 * offered because the blocks are what it draws, and no colour, because
 * pixelation takes its colour from around the box. */
export const Redaction: Story = {
  args: { tool: "annotation", workspace: "screenshot" },
  beforeEach: () => {
    seed(
      {
        annotation: {
          animated: false,
          id: "redact-1",
          kind: "redact",
          style: {
            ...dress,
            color: "#000000",
            redaction: "pixelate",
            width: 12,
          },
        },
        cursorEffects: DEFAULT_CURSOR_EFFECTS,
        frame: null,
        isLocked: false,
        selection: null,
      },
      "screenshot",
    );
  },
};

/** The same box pixelated the classic way: ordinary blocks, which the note
 * says are a look rather than a way to hide anything sensitive. */
export const RedactionClassic: Story = {
  args: { tool: "annotation", workspace: "screenshot" },
  beforeEach: () => {
    seed(
      {
        annotation: {
          animated: false,
          id: "redact-1",
          kind: "redact",
          style: {
            ...dress,
            color: "#000000",
            redaction: "pixelateClassic",
            width: 12,
          },
        },
        cursorEffects: DEFAULT_CURSOR_EFFECTS,
        frame: null,
        isLocked: false,
        selection: null,
      },
      "screenshot",
    );
  },
};

/** The Redaction panel, blurring a face: a strength instead of a block size,
 * and corners rounded as far as they go, a circle over a square box. */
export const RedactionBlur: Story = {
  args: { tool: "annotation", workspace: "screenshot" },
  beforeEach: () => {
    seed(
      {
        annotation: {
          animated: false,
          id: "redact-1",
          kind: "redact",
          style: {
            ...dress,
            color: "#000000",
            radius: 50,
            redaction: "blur",
            strength: 3,
            width: 12,
          },
        },
        cursorEffects: DEFAULT_CURSOR_EFFECTS,
        frame: null,
        isLocked: false,
        selection: null,
      },
      "screenshot",
    );
  },
};

/** The Arrow panel: the annotation the preview has in hand, and what it is
 * drawn in. It follows the chosen arrow rather than a tool, so it is the one
 * panel that comes up over another. */
export const Arrow: Story = {
  args: { tool: "annotation", workspace: "recording" },
  beforeEach: () => {
    seed({
      annotation: {
        animated: true,
        id: "arrow-1",
        kind: "arrow",
        style: { ...dress, color: "#ff383c", head: "end", width: 8 },
      },
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      isLocked: false,
      selection: null,
    });
  },
};

/** The same panel wearing a colour of its own: colours chosen from the system
 * panel are kept after the palette, and the Custom tile is the chosen one
 * only while the annotation matches no tile at all. */
export const ArrowCustomColour: Story = {
  args: { tool: "annotation", workspace: "recording" },
  beforeEach: () => {
    seed({
      annotation: {
        animated: true,
        id: "arrow-1",
        kind: "arrow",
        style: { ...dress, color: "#2ec4b6", head: "both", width: 16 },
      },
      annotationColors: ["#2ec4b6", "#8b5e34"],
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      isLocked: false,
      selection: null,
    });
  },
};

/** Animate switched off: the annotation stands on screen for the whole of its
 * clip instead of drawing itself in and out at its ends. */
export const ArrowWithoutAnimation: Story = {
  args: { tool: "annotation", workspace: "recording" },
  beforeEach: () => {
    seed({
      annotation: {
        animated: false,
        id: "arrow-1",
        kind: "arrow",
        style: { ...dress, color: "#ff383c", head: "end", width: 8 },
      },
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      isLocked: false,
      selection: null,
    });
  },
};

/** The same panel in the screenshot editor. A still has no clip for an
 * annotation to arrive over, so the Animate row is not offered at all and the
 * panel is the three it has always been. */
export const ArrowInAScreenshot: Story = {
  args: { tool: "annotation", workspace: "screenshot" },
  beforeEach: () => {
    seed(
      {
        annotation: {
          animated: true,
          id: "arrow-1",
          kind: "arrow",
          style: { ...dress, color: "#ff383c", head: "end", width: 8 },
        },
        cursorEffects: DEFAULT_CURSOR_EFFECTS,
        frame: null,
        isLocked: false,
        selection: null,
      },
      "screenshot",
    );
  },
};
