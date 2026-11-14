// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { browseBackgroundImage } from "../../api";
import { presetName } from "../../tools/frame/background/background";
import { frameIsland } from "../frame/frame-island";

import type { BackgroundPreset } from "../../tools/frame/background/background";
import type { ToolPanelPatch } from "../../tools/tool-panel-patch";
import type { ToolPanelSnapshot } from "../../tools/tool-panel-store";
import type { EditorIslandContent } from "../editor-island";

/** The canvas: its size and background, over a scene that scene's
 * background, and for a screenshot its corners. A video's corners are never
 * rounded, so a recording offers none. */
export function frameIslandContent({
  capture,
  change,
  presets,
  snapshot,
}: {
  capture: "recording" | "screenshot";
  change: (values: ToolPanelPatch) => void;
  /** The backgrounds the app ships. */
  presets: BackgroundPreset[];
  snapshot: ToolPanelSnapshot;
}): EditorIslandContent | null {
  const { frame } = snapshot;
  if (!frame) return null;
  // Over a scene its background is the scene's own, the recording's where it
  // has none; the size stays the recording's.
  const scene = snapshot.scene?.hasSceneAtPlayhead ? snapshot.scene : null;
  return frameIsland({
    background: scene?.background ?? snapshot.background,
    height: frame.height,
    onBackgroundChange: (background) => {
      change(scene ? { sceneBackground: background } : { background });
    },
    onOriginalSize: () => {
      change({ resetFrame: true });
    },
    onPickImage: browseBackgroundImage,
    onRemovePreset: (id) => {
      change({ removePreset: id });
    },
    onSavePreset: (background, name) => {
      change({
        savePreset: {
          background,
          id: `saved-${Date.now().toString(36)}`,
          name: name ?? presetName(background),
        },
      });
    },
    onSizeChange: (frameSize) => {
      change({ frameSize });
    },
    presets,
    savedPresets: snapshot.backgroundPresets,
    width: frame.width,
    ...(capture === "screenshot"
      ? {
          onRadiusChange: (frameRadius: number) => {
            change({ frameRadius });
          },
          radius: frame.radius ?? 0,
        }
      : {}),
    ...(snapshot.scene?.hasScenes
      ? {
          scene: {
            hasOwnBackground: (scene?.background ?? null) !== null,
            isInScene: scene !== null,
            onClearSceneBackground: () => {
              change({ sceneBackground: null });
            },
          },
        }
      : {}),
  });
}
