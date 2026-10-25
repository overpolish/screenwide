// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { featureStage } from "../../../storybook/feature-story-stage";
import { toolPanelWidth } from "../../popup-panel/layout";
import { DEFAULT_CURSOR_EFFECTS } from "../export/recording-export-settings";

import { ToolPanel } from "./tool-panel";
import {
  ToolPanelCameraEffects,
  ToolPanelCameraObjects,
  ToolPanelMicrophone,
} from "./tool-panel-store";
import { seedToolPanel } from "./tool-panel-story-seed";

import type { Meta, StoryObj } from "@storybook/react-vite";

const seed = seedToolPanel;

const meta = {
  args: { tool: "selection", workspace: "recording" },
  component: ToolPanel,
  decorators: [featureStage({ width: toolPanelWidth })],
  parameters: { layout: "fullscreen" },
  title: "Legacy/Editor/Tool Panels/Selection",
} satisfies Meta<typeof ToolPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

/** The screen track of a recording, at the size and position the preview
 * would be showing it, padded out past its own picture and turned in depth by
 * its own turn, each dial aimed as the gizmo's ring of its colour stands. */
export const Selection: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    seed({
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasCursorData: true,
      isLocked: false,
      selection: {
        dropShadow: true,
        height: 2458,
        inset: 60,
        insetMaximum: 2338,
        kind: "primary",
        label: "Screen",
        radius: 8,
        rotation: { x: 15, y: -20, z: 0 },
        sourceHeight: 2338,
        sourceWidth: 3600,
        width: 3720,
        x: -60,
        y: -60,
      },
    });
  },
};

/** The screen inside a scene that turns it in depth: the Rotation row under
 * the radius, each dial aimed as the gizmo's ring of its colour stands. */
export const SelectionInTurnedScene: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    seed({
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasCursorData: true,
      isLocked: false,
      scene: {
        boxes: null,
        cameraAspect: 16 / 9,
        canvasAspect: 16 / 9,
        composition: {
          camera: { height: 0.22, width: 0.124, x: 0.84, y: 0.72 },
          screen: { height: 0.889, width: 0.8, x: 0.1, y: 0.056 },
        },
        followsFace: false,
        framing: { focusX: 0.5, focusY: 0.5, zoom: 1.5 },
        framingPane: "screen",
        hasAutoZooms: false,
        hasCamera: true,
        hasSceneAtPlayhead: true,
        isBaked: true,
        preset: "full",
        radius: 8,
        rotation: { x: -30, y: 25, z: 10 },
        screenAspect: 16 / 10,
        variant: {},
      },
      selection: {
        dropShadow: true,
        height: 2338,
        inset: 0,
        insetMaximum: 2338,
        kind: "primary",
        label: "Screen",
        radius: 8,
        sourceHeight: 2338,
        sourceWidth: 3600,
        width: 3600,
        x: 0,
        y: 0,
      },
    });
  },
};

/** The camera track of a recording saved as a file of its own: it is placed
 * and padded the way any other layer is. */
export const SelectionCamera: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    seed({
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasCursorData: true,
      isLocked: false,
      selection: {
        dropShadow: true,
        height: 720,
        inset: 0,
        insetMaximum: 720,
        kind: "camera",
        label: "Camera",
        radius: 8,
        sourceHeight: 720,
        sourceWidth: 1280,
        width: 1280,
        x: 2200,
        y: 1500,
      },
    });
  },
};

/** The same camera drawn into the screen's picture: the overlay is what is
 * placed, so the size is the window it is drawn in and there is no pad. It
 * can follow the face. */
export const SelectionBakedCamera: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    seed({
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasCursorData: true,
      isLocked: false,
      selection: {
        dropShadow: true,
        followsFace: true,
        height: 506,
        inset: 0,
        insetMaximum: 0,
        kind: "camera",
        label: "Camera",
        radius: 8,
        sourceHeight: 720,
        sourceWidth: 1280,
        width: 900,
        x: 2592,
        y: 73,
      },
    });
  },
};

const BAKED_CAMERA = {
  dropShadow: false,
  followsFace: false,
  height: 506,
  inset: 0,
  insetMaximum: 0,
  kind: "camera",
  label: "Camera",
  radius: 8,
  sourceHeight: 720,
  sourceWidth: 1280,
  width: 900,
  x: 2592,
  y: 73,
} as const;

const MODEL_BYTES = 14_975_696;

const cameraEffects = (
  effects: Partial<ToolPanelCameraEffects>,
): ToolPanelCameraEffects => ({
  background: "kept",
  enhance: false,
  hasImage: false,
  model: { progress: 0, sizeBytes: MODEL_BYTES, status: "downloaded" },
  objects: null,
  progress: { background: 0, colour: 0 },
  working: { background: false, colour: false },
  ...effects,
});

const cameraStory = (effects: Partial<ToolPanelCameraEffects>): Story => ({
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    seed({
      isLocked: false,
      selection: { ...BAKED_CAMERA, cameraEffects: cameraEffects(effects) },
    });
  },
});

/** A camera before the background removal model is here: a download in the
 * switch's place. */
export const SelectionCameraDownload = cameraStory({
  model: { progress: 0, sizeBytes: MODEL_BYTES, status: "available" },
});

/** The person being cut out of the room, with its bar. */
export const SelectionCameraRemovingBackground = cameraStory({
  background: "transparent",
  progress: { background: 0.4, colour: 0 },
  working: { background: true, colour: false },
});

/** The person in front of a picture, Enhance measuring their colour. */
export const SelectionCameraPicture = cameraStory({
  background: "image",
  enhance: true,
  hasImage: true,
  progress: { background: 0, colour: 0.7 },
  working: { background: false, colour: true },
});

/** The room behind the person blurred. */
export const SelectionCameraBlurred = cameraStory({
  background: "blurred",
});

const OBJECTS_MODEL_BYTES = 190_082_794;

const cameraObjects = (
  objects: Partial<ToolPanelCameraObjects>,
): ToolPanelCameraObjects => ({
  chosen: null,
  isPicking: false,
  model: { progress: 0, sizeBytes: OBJECTS_MODEL_BYTES, status: "downloaded" },
  picks: [],
  progress: 0,
  working: false,
  ...objects,
});

/** A matted camera before the object model is here: a download for it. */
export const SelectionCameraObjectsDownload = cameraStory({
  background: "transparent",
  objects: cameraObjects({
    model: { progress: 0, sizeBytes: OBJECTS_MODEL_BYTES, status: "available" },
  }),
});

/** More objects than the row holds, scrolled to the one chosen. */
export const SelectionCameraObjects = cameraStory({
  background: "transparent",
  objects: cameraObjects({
    chosen: 7,
    isPicking: true,
    picks: Array.from({ length: 8 }, (_, id) => ({
      followed: true,
      following: null,
      id,
      keep: id % 3 !== 0,
      painted: id % 4 === 1,
    })),
  }),
});

/** Picking: a painted kept object chosen and followed, one waiting. */
export const SelectionCameraPickingObjects = cameraStory({
  background: "transparent",
  objects: cameraObjects({
    chosen: 1,
    isPicking: true,
    picks: [
      { followed: true, following: null, id: 0, keep: false, painted: false },
      { followed: false, following: 0.35, id: 1, keep: true, painted: true },
      { followed: false, following: null, id: 2, keep: false, painted: false },
    ],
  }),
});

/** A system audio track picked out with the Select tool: nothing of it is
 * placed, and it carries no speech, so the panel offers only how loud it is
 * played back. */
export const SelectionAudio: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    seed({
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasCursorData: true,
      isLocked: false,
      selection: { decibels: 6, kind: "audio", label: "System audio" },
    });
  },
};

const STUDIO_MODEL_BYTES = 1_005_394_589;
const DUCKING_MODEL_BYTES = 165_612_636;

const microphone = (
  {
    autoVolume,
    ducking = null,
    noise,
    studio = "on",
    studioModel = {
      progress: 0,
      sizeBytes: STUDIO_MODEL_BYTES,
      status: "available",
    },
    voice,
  }: Pick<ToolPanelMicrophone, "autoVolume" | "noise" | "voice"> &
    Partial<Pick<ToolPanelMicrophone, "ducking" | "studio" | "studioModel">>,
  silences: Omit<ToolPanelMicrophone["silences"], "progress">,
  progress = { autoVolume: 0, noise: 0, silences: 0, studio: 0, voice: 0 },
) => {
  seed({
    cursorEffects: DEFAULT_CURSOR_EFFECTS,
    frame: null,
    hasCursorData: true,
    isLocked: false,
    selection: {
      decibels: 0,
      kind: "audio",
      label: "Microphone",
      microphone: {
        autoVolume,
        autoVolumeProgress: progress.autoVolume,
        ducking,
        noise,
        noiseProgress: progress.noise,
        silences: { ...silences, progress: progress.silences },
        studio,
        studioModel,
        studioProgress: progress.studio,
        voice,
        voiceProgress: progress.voice,
      },
    },
  });
};

/** A microphone before any speech tool has run. */
export const SelectionMicrophone: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    microphone(
      { autoVolume: "off", noise: "off", voice: "off" },
      { count: 0, durationMs: 0, status: "idle" },
    );
  },
};

/** The track being cleaned, its voice cleaned up, its volume measured and
 * the pauses being found, each with a bar for how far it has got. */
export const SelectionMicrophoneListening: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    microphone(
      { autoVolume: "cleaning", noise: "cleaning", voice: "cleaning" },
      { count: 0, durationMs: 0, status: "finding" },
      { autoVolume: 0.6, noise: 0.45, silences: 0.8, studio: 0, voice: 0.2 },
    );
  },
};

/** Every tool on and the pauses cut, which Restore all brings back. The
 * summary at its longest, a three-figure count and over an hour cut, still
 * fits the panel on one line. */
export const SelectionMicrophoneCleaned: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    microphone(
      { autoVolume: "on", noise: "on", voice: "on" },
      { count: 128, durationMs: 3_735_000, status: "idle" },
    );
  },
};

/** Speech with hardly a pause: Remove found nothing long enough to cut. */
export const SelectionMicrophoneNoPauses: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    microphone(
      { autoVolume: "off", noise: "off", voice: "off" },
      { count: 0, durationMs: 0, status: "none-found" },
    );
  },
};

/** The Studio sound model coming down, its bar under the row; Reduce noise
 * and Vocal cleanup stay until it is here. */
export const SelectionMicrophoneDownloadingStudioSound: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    microphone(
      {
        autoVolume: "on",
        noise: "on",
        studioModel: {
          progress: 0.35,
          sizeBytes: STUDIO_MODEL_BYTES,
          status: "downloading",
        },
        voice: "on",
      },
      { count: 0, durationMs: 0, status: "idle" },
    );
  },
};

/** Studio sound rebuilding the voice, in place of Reduce noise and Vocal
 * cleanup. */
export const SelectionMicrophoneStudioSound: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    microphone(
      {
        autoVolume: "on",
        noise: "on",
        studio: "cleaning",
        studioModel: {
          progress: 0,
          sizeBytes: STUDIO_MODEL_BYTES,
          status: "downloaded",
        },
        voice: "on",
      },
      { count: 0, durationMs: 0, status: "idle" },
      { autoVolume: 0, noise: 0, silences: 0, studio: 0.4, voice: 0 },
    );
  },
};

/** A recording with system audio, before the Studio ducking model is here:
 * a download under Auto volume, whose ducking it changes. */
export const SelectionMicrophoneStudioDuckingDownload: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    microphone(
      {
        autoVolume: "on",
        ducking: {
          model: {
            progress: 0,
            sizeBytes: DUCKING_MODEL_BYTES,
            status: "available",
          },
          progress: 0,
          state: "on",
        },
        noise: "on",
        voice: "on",
      },
      { count: 0, durationMs: 0, status: "idle" },
    );
  },
};

/** Studio ducking finding the singing in the system audio, with its bar. */
export const SelectionMicrophoneStudioDucking: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    microphone(
      {
        autoVolume: "on",
        ducking: {
          model: {
            progress: 0,
            sizeBytes: DUCKING_MODEL_BYTES,
            status: "downloaded",
          },
          progress: 0.55,
          state: "cleaning",
        },
        noise: "on",
        voice: "on",
      },
      { count: 0, durationMs: 0, status: "idle" },
    );
  },
};

/** The tool is in hand with nothing under it: the panel says so rather than
 * offering fields that would place nothing. */
export const SelectionEmpty: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    seed({
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasCursorData: true,
      isLocked: false,
      selection: null,
    });
  },
};
