// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { browseBackgroundImage } from "../../api";
import { presetName } from "../../tools/frame/background/background";
import { frameIsland } from "../frame/frame-island";

import type {
  Background,
  BackgroundPreset,
} from "../../tools/frame/background/background";
import type { ToolPanelPatch } from "../../tools/tool-panel-patch";
import type { ToolPanelSnapshot } from "../../tools/tool-panel-store";
import type { EditorIslandContent } from "../editor-island";

/** What every island choosing a background asks for besides the background
 * itself: a picture of your own, and the tiles kept and forgotten, which
 * the Frame island and a gap's share. */
export const backgroundChoices = (
  change: (values: ToolPanelPatch) => void,
  {
    presets,
    snapshot,
  }: { presets: BackgroundPreset[]; snapshot: ToolPanelSnapshot },
) => ({
  onPickImage: browseBackgroundImage,
  onRemovePreset: (id: string) => {
    change({ removePreset: id });
  },
  onSavePreset: (background: Background, name?: string) => {
    change({
      savePreset: {
        background,
        id: `saved-${Date.now().toString(36)}`,
        name: name ?? presetName(background),
      },
    });
  },
  presets,
  savedPresets: snapshot.backgroundPresets,
});

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
  const scene = snapshot.scene?.hasSceneInHand ? snapshot.scene : null;
  return frameIsland({
    ...backgroundChoices(change, { presets, snapshot }),
    background: scene?.background ?? snapshot.background,
    height: frame.height,
    onBackgroundChange: (background) => {
      change(scene ? { sceneBackground: background } : { background });
    },
    onOriginalSize: () => {
      change({ resetFrame: true });
    },
    onSizeChange: (frameSize) => {
      change({ frameSize });
    },
    ...(frame.formats ? { formats: frame.formats } : {}),
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
