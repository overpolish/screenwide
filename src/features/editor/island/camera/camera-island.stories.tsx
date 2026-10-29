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

import { cameraIsland, type CameraIslandProps } from "./camera-island";

import type {
  ToolPanelCameraEffects,
  ToolPanelCameraObjects,
  ToolPanelModel,
} from "../../tool-panels/tool-panel-selection";
import type { Meta, StoryObj } from "@storybook/react-vite";

const READY: ToolPanelModel = {
  progress: 1,
  sizeBytes: 46_000_000,
  status: "downloaded",
};

const NO_OBJECTS: ToolPanelCameraObjects = {
  chosen: null,
  isPicking: false,
  model: { progress: 1, sizeBytes: 152_000_000, status: "downloaded" },
  picks: [],
  progress: 0,
  working: false,
};

const EFFECTS: ToolPanelCameraEffects = {
  background: "kept",
  enhance: false,
  hasImage: false,
  model: READY,
  objects: null,
  progress: { background: 0, colour: 0 },
  working: { background: false, colour: false },
};

/** A kept mug, a removed lamp and a painted line, the last waiting its
 * turn to be followed. */
const PICKS: ToolPanelCameraObjects["picks"] = [
  { followed: true, following: null, id: 1, keep: true, painted: false },
  { followed: true, following: null, id: 2, keep: false, painted: false },
  { followed: false, following: null, id: 3, keep: true, painted: true },
];

function CameraIsland({
  isShown,
  ...props
}: EditorIslandStoryProps & CameraIslandProps) {
  return <EditorIslandStage content={cameraIsland(props)} isShown={isShown} />;
}

const meta = {
  ...editorIslandStoryMeta,
  argTypes: {
    aspect: {
      control: "inline-radio",
      options: ["free", "original", "1:1", "4:3", "16:9", "9:16"],
      table: { defaultValue: { summary: "free" } },
    },
  },
  args: {
    aspect: "free",
    effects: EFFECTS,
    followFace: false,
    hasShadow: false,
    isFindingFace: false,
    isInScene: false,
    isShown: true,
    onAspectChange: fn(),
    onBackgroundChange: fn(),
    onChoosePicture: fn(),
    onDownloadBackgroundModel: fn(),
    onDownloadObjectsModel: fn(),
    onEnhanceChange: fn(),
    onFollowFaceChange: fn(),
    onPickObjects: fn(),
    onRadiusChange: fn(),
    onRemoveObject: fn(),
    onReset: fn(),
    onShadowChange: fn(),
    onShowObject: fn(),
    radius: 50,
  },
  component: CameraIsland,
  decorators: [featureStage({ height: 420, width: 520 })],
  // Every choice changes the story's camera, as it would the recording's. A
  // story cannot download a model or open a picker, so a download lands at
  // once and a picture is chosen as soon as it is asked for.
  render: function Render(args) {
    const [, updateArgs] = useArgs<CameraIslandProps>();
    const { effects } = args;
    const objects = effects.objects ?? NO_OBJECTS;
    const change = (next: Partial<ToolPanelCameraEffects>) => {
      updateArgs({ effects: { ...effects, ...next } });
    };
    return (
      <CameraIsland
        {...args}
        onAspectChange={(aspect) => {
          args.onAspectChange(aspect);
          updateArgs({ aspect });
        }}
        onBackgroundChange={(background) => {
          args.onBackgroundChange(background);
          change({ background, objects: effects.objects ?? NO_OBJECTS });
        }}
        onChoosePicture={() => {
          args.onChoosePicture();
          change({ background: "image", hasImage: true });
        }}
        onDownloadBackgroundModel={() => {
          args.onDownloadBackgroundModel();
          change({ model: READY });
        }}
        onDownloadObjectsModel={() => {
          args.onDownloadObjectsModel();
          change({ objects: { ...objects, model: NO_OBJECTS.model } });
        }}
        onEnhanceChange={(enhance) => {
          args.onEnhanceChange(enhance);
          change({ enhance });
        }}
        onFollowFaceChange={(followFace) => {
          args.onFollowFaceChange(followFace);
          updateArgs({ followFace });
        }}
        onPickObjects={(isPicking) => {
          args.onPickObjects(isPicking);
          change({ objects: { ...objects, isPicking } });
        }}
        onRadiusChange={(radius) => {
          args.onRadiusChange(radius);
          updateArgs({ radius });
        }}
        onRemoveObject={(id) => {
          args.onRemoveObject(id);
          change({
            objects: {
              ...objects,
              chosen: null,
              picks: objects.picks.filter((pick) => pick.id !== id),
            },
          });
        }}
        onReset={() => {
          args.onReset();
          updateArgs({ aspect: "free" });
        }}
        onShadowChange={(hasShadow) => {
          args.onShadowChange(hasShadow);
          updateArgs({ hasShadow });
        }}
        onShowObject={(id) => {
          args.onShowObject(id);
          change({ objects: { ...objects, chosen: id } });
        }}
      />
    );
  },
  title: "Features/Editor/Island/Camera",
} satisfies Meta<typeof CameraIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// Over a scene, the corners are the scene's own; the note says so.
export const InScene: Story = { args: { isInScene: true } };

export const FindingFace: Story = {
  args: { followFace: true, isFindingFace: true },
};

export const NeedsModel: Story = {
  args: {
    effects: {
      ...EFFECTS,
      model: { progress: 0, sizeBytes: 46_000_000, status: "available" },
    },
  },
};

export const DownloadingModel: Story = {
  args: {
    effects: {
      ...EFFECTS,
      model: { progress: 0.4, sizeBytes: 46_000_000, status: "downloading" },
    },
  },
};

// The cutout and the colour measure both being made, the first time each
// is asked for.
export const Working: Story = {
  args: {
    effects: {
      ...EFFECTS,
      background: "transparent",
      enhance: true,
      objects: NO_OBJECTS,
      progress: { background: 0.6, colour: 0.25 },
      working: { background: true, colour: true },
    },
  },
};

export const BackgroundBlurred: Story = {
  args: {
    effects: {
      ...EFFECTS,
      background: "blurred",
      objects: { ...NO_OBJECTS, picks: PICKS },
    },
  },
};

export const BackgroundPicture: Story = {
  args: {
    effects: {
      ...EFFECTS,
      background: "image",
      hasImage: true,
      objects: NO_OBJECTS,
    },
  },
};

export const ObjectsNeedModel: Story = {
  args: {
    effects: {
      ...EFFECTS,
      background: "transparent",
      objects: {
        ...NO_OBJECTS,
        model: { progress: 0, sizeBytes: 152_000_000, status: "available" },
      },
    },
  },
};

// Picking on the preview, one object chosen and another being followed
// through the recording.
export const PickingObjects: Story = {
  args: {
    effects: {
      ...EFFECTS,
      background: "transparent",
      objects: {
        ...NO_OBJECTS,
        chosen: 2,
        isPicking: true,
        picks: [
          ...PICKS,
          {
            followed: false,
            following: 0.45,
            id: 4,
            keep: false,
            painted: false,
          },
        ],
        progress: 0.3,
        working: true,
      },
    },
  },
};

// More objects than the strip shows at once: the marks scroll rather than
// widening the island.
export const ManyObjects: Story = {
  args: {
    effects: {
      ...EFFECTS,
      background: "transparent",
      objects: {
        ...NO_OBJECTS,
        chosen: 8,
        picks: Array.from({ length: 10 }, (_, index) => ({
          followed: true,
          following: null,
          id: index + 1,
          keep: index % 3 !== 1,
          painted: index % 4 === 3,
        })),
      },
    },
  },
};
