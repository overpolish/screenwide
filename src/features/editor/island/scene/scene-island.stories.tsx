// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { EditorIslandStage } from "../editor-island-stage";
import {
  editorIslandStoryMeta,
  type EditorIslandStoryProps,
} from "../editor-island-story-meta";

import { sceneIsland, type SceneIslandProps } from "./scene-island";
import {
  CUSTOM,
  FRAMING_ARGS,
  framingHandlers,
  MANY_SAVED_LAYOUTS,
  SAVED_LAYOUT,
  TALL_VIDEO,
} from "./scene-story-fixtures";

import type { Meta, StoryObj } from "@storybook/react-vite";

function SceneIsland({
  isShown,
  ...props
}: SceneIslandProps & EditorIslandStoryProps) {
  return <EditorIslandStage content={sceneIsland(props)} isShown={isShown} />;
}

const meta = {
  ...editorIslandStoryMeta,
  args: {
    ...FRAMING_ARGS,
    autoZoom: { hasAutoZooms: true },
    group: "layout",
    isShown: true,
    onClearAutoZooms: fn(),
    onDelete: fn(),
    onGroupChange: fn(),
    onRegenerateAutoZooms: fn(),
    scene: { id: "scene", isPaused: false },
  },
  component: SceneIsland,
  decorators: [featureStage({ height: 440, width: 520 })],
  // Every choice changes the story's args, so groups open and close, the
  // layout and its options change, and the figures follow, as they would
  // in the editor.
  render: function Render(args) {
    const [, updateArgs] = useArgs<SceneIslandProps>();
    return (
      <SceneIsland
        {...args}
        {...framingHandlers(args, updateArgs)}
        onClearAutoZooms={() => {
          args.onClearAutoZooms();
          updateArgs({ autoZoom: { hasAutoZooms: false } });
        }}
        onGroupChange={(group) => {
          args.onGroupChange(group);
          updateArgs({ group });
        }}
        onRegenerateAutoZooms={() => {
          args.onRegenerateAutoZooms();
          updateArgs({ autoZoom: { hasAutoZooms: true } });
        }}
      />
    );
  },
  title: "Features/Editor/Island/Scene",
} satisfies Meta<typeof SceneIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const SideBySide: Story = {};

export const PictureInPicture: Story = {
  args: { layout: { kind: "preset", preset: "picture-in-picture" } },
};

// A 1080 by 1920 video: every layout is drawn on its shape.
export const TallVideo: Story = { args: { video: TALL_VIDEO } };

export const CameraOnly: Story = {
  args: { layout: { kind: "preset", preset: "camera-only" }, screen: null },
};

export const Custom: Story = {
  args: { layout: { boxes: CUSTOM, kind: "custom" } },
};

export const SavedLayout: Story = {
  args: {
    layout: { boxes: SAVED_LAYOUT.boxes, id: SAVED_LAYOUT.id, kind: "saved" },
  },
};

export const ManySavedLayouts: Story = {
  args: { savedLayouts: MANY_SAVED_LAYOUTS },
};

export const Camera: Story = { args: { group: "camera" } };

export const FindingFace: Story = {
  args: {
    camera: {
      focusX: 0.5,
      focusY: 0.4,
      followFace: true,
      hasOwnRadius: false,
      isFindingFace: true,
      radius: 50,
      zoom: 1.6,
    },
    group: "camera",
  },
};

export const Screen: Story = {
  args: {
    group: "screen",
    screen: {
      focusX: 0.4,
      focusY: 0.5,
      hasOwnRadius: true,
      radius: 12,
      rotation: { x: 0, y: -20, z: 0 },
      zoom: 2,
    },
  },
};

// A recording without a camera, or with it exported on its own: only a
// layout of the screen's own can be chosen.
export const WithoutCamera: Story = {
  args: {
    camera: null,
    canPlaceCamera: false,
    layout: { boxes: { screen: CUSTOM.screen }, kind: "custom" },
    savedLayouts: [],
  },
};

// The scene places the camera, which is exported on its own, so only
// layouts without the camera can be chosen.
export const Paused: Story = {
  args: { canPlaceCamera: false, scene: { id: "scene", isPaused: true } },
};

// The tool with no scene in hand frames the recording itself.
export const Global: Story = { args: { scene: null } };

export const AutoZoom: Story = { args: { group: "autoZoom", scene: null } };

export const AutoZoomCleared: Story = {
  args: { autoZoom: { hasAutoZooms: false }, group: "autoZoom", scene: null },
};
