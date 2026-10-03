// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useSyncExternalStore } from "react";

import { ToolPanelScene } from "../../tool-panels/tool-panel-scene";
import { EditorKind } from "../../types";

import { ScenePane } from "./recording-scene-order";
import { SceneTemplate, SceneTemplateLayout } from "./recording-scene-template";
import { SceneVariant } from "./recording-scene-variant";
import { RecordingScenePreset, SceneFraming } from "./recording-scenes";

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
  choosePreset: (preset: RecordingScenePreset) => void;
  /** Lay the scene out as `template`, fitted to this canvas. */
  chooseTemplate: (template: SceneTemplate) => void;
  /** Make the scene custom, its panes kept where it puts them now. */
  customize: () => void;
  remove: () => void;
  /** Show the whole screen and camera again, in the preset's own boxes. */
  resetFraming: () => void;
  /** Zoom into the selected pane, or move the part of it shown, a field at a
   * time. */
  setFraming: (framing: Partial<SceneFraming>) => void;
  /** Round the selected pane's corners, in percent of its shorter side. */
  setRadius: (radius: number) => void;
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
        choosePreset: (next) => {
          controlsRef.current.choosePreset(next);
        },
        chooseTemplate: (template) => {
          controlsRef.current.chooseTemplate(template);
        },
        customize: () => {
          controlsRef.current.customize();
        },
        remove: () => {
          controlsRef.current.remove();
        },
        resetFraming: () => {
          controlsRef.current.resetFraming();
        },
        setFraming: (framing) => {
          controlsRef.current.setFraming(framing);
        },
        setRadius: (radius) => {
          controlsRef.current.setRadius(radius);
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
