// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useSyncExternalStore } from "react";

import { type CropAspect } from "../../island/crop-aspect-choice";
import { ToolPanelScene } from "../../tool-panels/tool-panel-scene";
import { EditorKind } from "../../types";

import { ScenePane } from "./recording-scene-order";
import { SceneChoice, SceneSpanTarget } from "./recording-scene-span";
import { SceneTemplate, SceneTemplateLayout } from "./recording-scene-template";
import { SceneVariant } from "./recording-scene-variant";
import {
  RecordingScenePreset,
  SceneFraming,
  SceneRotation,
} from "./recording-scenes";

import type { Arrangement } from "../../annotations/annotation-order";

/**
 * The scene under the preview's playhead, and the edits that change it,
 * reachable from the Scene panel.
 *
 * Where the playhead stands is the preview's own business, while the panel's
 * asks arrive through the editor's bridge; the preview leaves what is under
 * its playhead here for the bridge to find, the way the selected keyboard
 * shortcut is published.
 */
export type RecordingSceneControls = {
  /** Move `pane` through the order of a custom scene's two panes. */
  arrangePane: (pane: ScenePane, move: Arrangement) => void;
  /** Make the auto zooms again from the recording's cursor and keys, in
   * place of the ones made before; your own scenes stay as they are. */
  autoZoom: () => void;
  /** Add the scene `target` names, or lay out and move the one it names, as
   * `choice`: what the Transcript panel's menu asks for the words chosen. */
  chooseForSpan: (target: SceneSpanTarget, choice: SceneChoice) => void;
  choosePreset: (preset: RecordingScenePreset) => void;
  /** Lay the scene out as `template`, fitted to this canvas. */
  chooseTemplate: (template: SceneTemplate) => void;
  /** Take every auto zoom off the timeline; your own scenes stay as they
   * are. */
  clearAutoZooms: () => void;
  /** Crop the scene's screen to `aspect`'s shape about its middle, the scene
   * made custom where it is a preset's; a free crop changes nothing. */
  cropScreen: (aspect: CropAspect) => void;
  /** Crop the scene's screen to the content found in what it shows, in the
   * middle of where its box was, the scene made custom where it is a
   * preset's. */
  recenterScreen: () => void;
  remove: () => void;
  /** Take the scene's crop of the screen away. */
  resetScreenCrop: () => void;
  /** Have the camera follow the face, which starts it in the middle of the
   * camera's box, or stop following it. */
  setFollowFace: (follow: boolean) => void;
  /** Zoom into `pane`, the selected pane where unset, or move the part of
   * it shown, a field at a time. */
  setFraming: (framing: Partial<SceneFraming>, pane?: ScenePane) => void;
  /** Pad the scene's screen by this share of its box's shorter side on
   * each side; none takes the padding away. */
  setInset: (inset: number) => void;
  /** Round the selected pane's corners, in percent of its shorter side. */
  setRadius: (radius: number) => void;
  /** Turn the screen in depth, in degrees about each axis, a field at a
   * time. */
  setRotation: (rotation: Partial<SceneRotation>) => void;
  /** Change the preset's options a field at a time. */
  setVariant: (variant: SceneVariant) => void;
  /** Trade a custom scene's two panes: boxes and order. */
  swapPanes: () => void;
  /** What a template would keep of the scene, null where it is not custom. */
  templateLayout: () => SceneTemplateLayout | null;
};

type PublishedScene = {
  controls: RecordingSceneControls;
  scene: ToolPanelScene | null;
};

const workspaces = new Map<EditorKind, PublishedScene>();
const listeners = new Set<() => void>();

const notify = () => {
  for (const listener of listeners) listener();
};

const subscribe = (listener: () => void) => {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
};

/** Publish this workspace's scene for as long as its preview is mounted.
 * `scene` is null where the recording has nothing to arrange. It is compared
 * by value: the preview rebuilds it every render, and a panel re-rendered on
 * identity alone would never settle. */
export function usePublishRecordingScene(
  workspace: EditorKind,
  scene: ToolPanelScene | null,
  controls: RecordingSceneControls,
) {
  const controlsRef = useRef(controls);
  controlsRef.current = controls;
  const serialized = scene === null ? null : JSON.stringify(scene);
  useEffect(() => {
    const published: PublishedScene = {
      // The controls are read through the ref when called, so a panel that
      // holds this object still reaches the preview's current edit.
      controls: {
        arrangePane: (pane, move) => {
          controlsRef.current.arrangePane(pane, move);
        },
        autoZoom: () => {
          controlsRef.current.autoZoom();
        },
        chooseForSpan: (target, choice) => {
          controlsRef.current.chooseForSpan(target, choice);
        },
        choosePreset: (next) => {
          controlsRef.current.choosePreset(next);
        },
        chooseTemplate: (template) => {
          controlsRef.current.chooseTemplate(template);
        },
        clearAutoZooms: () => {
          controlsRef.current.clearAutoZooms();
        },
        cropScreen: (aspect) => {
          controlsRef.current.cropScreen(aspect);
        },
        recenterScreen: () => {
          controlsRef.current.recenterScreen();
        },
        remove: () => {
          controlsRef.current.remove();
        },
        resetScreenCrop: () => {
          controlsRef.current.resetScreenCrop();
        },
        setFollowFace: (follow) => {
          controlsRef.current.setFollowFace(follow);
        },
        setFraming: (framing, pane) => {
          controlsRef.current.setFraming(framing, pane);
        },
        setInset: (inset) => {
          controlsRef.current.setInset(inset);
        },
        setRadius: (radius) => {
          controlsRef.current.setRadius(radius);
        },
        setRotation: (rotation) => {
          controlsRef.current.setRotation(rotation);
        },
        setVariant: (variant) => {
          controlsRef.current.setVariant(variant);
        },
        swapPanes: () => {
          controlsRef.current.swapPanes();
        },
        templateLayout: () => controlsRef.current.templateLayout(),
      },
      scene:
        serialized === null ? null : (JSON.parse(serialized) as ToolPanelScene),
    };
    workspaces.set(workspace, published);
    notify();
    return () => {
      if (workspaces.get(workspace) !== published) return;
      workspaces.delete(workspace);
      notify();
    };
  }, [serialized, workspace]);
}

export const useRecordingSceneSelection = (workspace: EditorKind) =>
  useSyncExternalStore(
    subscribe,
    () => workspaces.get(workspace)?.scene ?? null,
  );

/** What the Scene panel's asks reach in `workspace`'s preview, while it is
 * mounted. */
export const recordingSceneControls = (workspace: EditorKind) =>
  workspaces.get(workspace)?.controls;
