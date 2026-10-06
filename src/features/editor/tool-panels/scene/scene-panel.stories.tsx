// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../../../storybook/feature-story-stage";
import { toolPanelWidth } from "../../../popup-panel/layout";
import { ToolPanel } from "../tool-panel";
import { seedToolPanel } from "../tool-panel-story-seed";

import type { Meta, StoryObj } from "@storybook/react-vite";

const seed = seedToolPanel;

const meta = {
  args: { tool: "scene", workspace: "recording" },
  component: ToolPanel,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage viewMode={context.viewMode} width={toolPanelWidth}>
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Editor/Tool Panel/Scene",
} satisfies Meta<typeof ToolPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

/** A 16:10 screen padded inside a 16:9 frame, with a square camera in its
 * corner: the recording's own composition a full scene keeps. */
const STORY_COMPOSITION = {
  camera: { height: 0.22, width: 0.124, x: 0.84, y: 0.72 },
  screen: { height: 0.889, width: 0.8, x: 0.1, y: 0.056 },
};

/** The fields each story's scene shares. */
const scene = {
  boxes: null,
  cameraAspect: 16 / 9,
  canvasAspect: 16 / 9,
  composition: STORY_COMPOSITION,
  framingPane: "screen",
  hasAutoZooms: false,
  hasCamera: true,
  isBaked: true,
  screenAspect: 16 / 10,
} as const;

/** The panel over a zoom: a custom scene kept in the recording's own boxes,
 * Custom filled with the accent, the zoom and the part of the screen it shows
 * following, then Auto zoom for a recording with cursor movement, with auto
 * zooms to clear, and the actions ending the panel. */
export const OverZoom: Story = {
  beforeEach: () => {
    seed({
      cameraOutput: "combined",
      hasCursorData: true,
      isLocked: false,
      scene: {
        ...scene,
        boxes: STORY_COMPOSITION,
        framing: { focusX: 0.3, focusY: 0.4, zoom: 2 },
        hasAutoZooms: true,
        hasSceneAtPlayhead: true,
        preset: "full",
        radius: 12,
        variant: {},
      },
    });
  },
};

/** The panel between scenes for a square frame: Custom is chosen, standing
 * for the recording's own composition, and each drawing takes the frame's
 * own shape, with the screen's own inside it. */
export const InSquareFrame: Story = {
  beforeEach: () => {
    seed({
      cameraOutput: "combined",
      isLocked: false,
      scene: {
        ...scene,
        canvasAspect: 1,
        composition: {
          camera: null,
          screen: { height: 0.5, width: 0.8, x: 0.1, y: 0.25 },
        },
        framing: null,
        hasSceneAtPlayhead: false,
        preset: null,
        radius: null,
        variant: null,
      },
    });
  },
};

/** A custom scene, its screen moved left and its camera grown beside it: the
 * Custom tile is chosen and draws the panes where the scene has them. */
export const Custom: Story = {
  beforeEach: () => {
    seed({
      cameraOutput: "combined",
      isLocked: false,
      scene: {
        ...scene,
        boxes: {
          camera: { height: 0.4, width: 0.225, x: 0.7, y: 0.3 },
          screen: { height: 0.6, width: 0.6, x: 0.05, y: 0.2 },
        },
        framing: { focusX: 0.5, focusY: 0.5, zoom: 1 },
        hasSceneAtPlayhead: true,
        preset: "split-two-thirds",
        radius: 8,
        variant: {},
      },
    });
  },
};

/** A picture in picture made custom and swapped: the camera fills the frame
 * behind a small screen in its corner, the Custom tile draws the screen over
 * it, and Swap trades them back. */
export const Swapped: Story = {
  beforeEach: () => {
    seed({
      cameraOutput: "combined",
      isLocked: false,
      scene: {
        ...scene,
        boxes: {
          camera: STORY_COMPOSITION.screen,
          cameraBehind: true,
          screen: { height: 0.3, width: 0.3, x: 0.65, y: 0.62 },
        },
        framing: { focusX: 0.5, focusY: 0.5, zoom: 1 },
        hasSceneAtPlayhead: true,
        preset: "picture-in-picture",
        radius: 12,
        variant: {},
      },
    });
  },
};

/** The custom scene above laid out as a saved template, beside one made on
 * a tall canvas: the template's tile is chosen, the tall one is drawn fitted
 * into this wide frame, and the custom scene can be kept as another. */
export const WithTemplates: Story = {
  beforeEach: () => {
    const boxes = {
      camera: { height: 0.4, width: 0.225, x: 0.7, y: 0.3 },
      screen: { height: 0.6, width: 0.6, x: 0.05, y: 0.2 },
    };
    seed({
      cameraOutput: "combined",
      isLocked: false,
      scene: {
        ...scene,
        boxes,
        framing: { focusX: 0.5, focusY: 0.5, zoom: 1 },
        hasSceneAtPlayhead: true,
        preset: "full",
        radius: 8,
        variant: {},
      },
      sceneTemplates: [
        { boxes, canvasAspect: 16 / 9, id: "beside", name: "Template 1" },
        {
          boxes: {
            camera: { height: 0.3, width: 0.6, x: 0.2, y: 0.08 },
            screen: { height: 0.3, width: 0.9, x: 0.05, y: 0.5 },
          },
          canvasAspect: 9 / 16,
          id: "tall",
          name: "Template 2",
        },
      ],
    });
  },
};

/** A side by side scene with the camera selected, swapped to the left at two
 * thirds: the tiles follow the options, the Custom tile wears the pencil
 * that says a press would free the layout's boxes, and the zoom and position
 * reach the camera. */
export const OverPresetWithCamera: Story = {
  beforeEach: () => {
    seed({
      cameraOutput: "combined",
      isLocked: false,
      scene: {
        ...scene,
        framing: { focusX: 0.6, focusY: 0.5, zoom: 1.5 },
        framingPane: "camera",
        hasSceneAtPlayhead: true,
        preset: "split-two-thirds",
        radius: 50,
        variant: { cameraSize: "two-thirds", swap: true },
      },
    });
  },
};

/** A picture in picture with a small camera over the top left corner: the
 * options become the corner and the size. */
export const PictureInPicture: Story = {
  beforeEach: () => {
    seed({
      cameraOutput: "combined",
      isLocked: false,
      scene: {
        ...scene,
        framing: { focusX: 0.5, focusY: 0.5, zoom: 1 },
        hasSceneAtPlayhead: true,
        preset: "picture-in-picture",
        radius: 12,
        variant: { corner: "top-left", size: "small" },
      },
    });
  },
};

/** Between scenes while the camera is saved as a file of its own: the output
 * heads the panel and nothing that places the camera is offered. */
export const SeparateFiles: Story = {
  beforeEach: () => {
    seed({
      cameraOutput: "separate",
      isLocked: false,
      scene: {
        ...scene,
        composition: { ...STORY_COMPOSITION, camera: null },
        framing: null,
        hasSceneAtPlayhead: false,
        isBaked: false,
        preset: null,
        radius: null,
        variant: null,
      },
    });
  },
};

/** A recording without a camera: every preset arranges one, so only Custom
 * is offered. */
export const WithoutCamera: Story = {
  beforeEach: () => {
    seed({
      isLocked: false,
      scene: {
        ...scene,
        composition: { ...STORY_COMPOSITION, camera: null },
        framing: null,
        hasCamera: false,
        hasSceneAtPlayhead: false,
        preset: null,
        radius: null,
        variant: null,
      },
    });
  },
};

/** Between scenes once every auto zoom is cleared: Regenerate makes them
 * again, and Clear all has nothing left to take off. */
export const AutoZoomsCleared: Story = {
  beforeEach: () => {
    seed({
      cameraOutput: "combined",
      hasCursorData: true,
      isLocked: false,
      scene: {
        ...scene,
        framing: null,
        hasSceneAtPlayhead: false,
        preset: null,
        radius: null,
        variant: null,
      },
    });
  },
};
