// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { Alert } from "../../../../primitives/alert/alert";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { Text } from "../../../../primitives/text/text";
import { SceneLayoutGlyph } from "../../glyphs/scene-layout-glyph";

import { SceneActions, type SceneToolGroup } from "./scene-actions";
import { SceneAutoZoomGroup } from "./scene-auto-zoom-group";
import {
  framingGroups,
  layoutDrawing,
  type SceneFramingProps,
} from "./scene-framing";
import { SceneLayoutGroup } from "./scene-layout-group";
import { ScenePaneGroup } from "./scene-pane-group";
import { SCENE_ISLAND_WIDTH } from "./scene-width";

import type { EditorIslandContent } from "../editor-island";

// The scene tool: how the picture is framed. With a scene in hand it frames
// that scene's stretch, and can delete it; with none, it frames the
// recording wherever no scene covers it, and holds the zooms planned from
// the cursor and keys. The row, drawn as the layout, chooses which group of
// options hangs beneath; one always does, and the one last chosen opens
// again for the next scene, or the layout where that group is missing.
// Nothing here adds a scene: one is drawn along the strip's scene line and
// set up once in hand, which the row's note says while none is. Moving,
// zooming and rounding a pane is done on the picture; the groups hold the
// choices, and exact figures for the framing.

export type SceneIslandProps = SceneFramingProps & {
  /** The planned zooms, where the recording has the cursor and keys to plan
   * them from; `hasAutoZooms` while any is left. */
  autoZoom: { hasAutoZooms: boolean } | null;
  /** The group open beneath the row, kept by the editor so it opens again
   * for the next scene. */
  group: SceneToolGroup;
  /** Removes every planned zoom; one changed by hand stays. */
  onClearAutoZooms: () => void;
  /** Deletes the scene in hand. */
  onDelete: () => void;
  onGroupChange: (group: SceneToolGroup) => void;
  /** Plans the zooms again, in place of those planned before. */
  onRegenerateAutoZooms: () => void;
  /** The scene in hand, or none, to frame the recording itself; `isPaused`
   * where the scene needs the camera, which is left out of the video. */
  scene: { id: string; isPaused: boolean } | null;
};

export function sceneIsland(props: SceneIslandProps): EditorIslandContent {
  const { autoZoom, scene } = props;
  // A paused scene can still be given a layout that needs no camera, and
  // its screen framed; its camera, out of the video, has nothing to frame.
  const groups: SceneToolGroup[] = scene?.isPaused
    ? framingGroups(props).filter((group) => group !== "camera")
    : [
        ...framingGroups(props),
        ...(scene === null && autoZoom ? (["autoZoom"] as const) : []),
      ];
  const open: SceneToolGroup | null = groups.includes(props.group)
    ? props.group
    : groups.length > 0
      ? groups[0]
      : null;
  return {
    actions: (
      <SceneActions
        groups={groups}
        onDelete={scene ? props.onDelete : undefined}
        onGroupChange={props.onGroupChange}
        open={open}
      />
    ),
    anchor: "end",
    // A line sits in the row's black, at the island's width, so the black
    // keeps one width whichever group is open, while staying within the
    // row's own width where the row is wider. Its
    // words start on the island's text column, with the rows' names below
    // and the title's glyph above. A paused scene says so as the island's
    // alerts do; with no scene in hand, a note says how to add one.
    below: scene?.isPaused ? (
      <Alert
        className={cn(SCENE_ISLAND_WIDTH.line, "px-inset pb-inset-sm")}
        role="status"
      >
        {t("editor-island-scene-paused")}
      </Alert>
    ) : scene === null ? (
      <Text
        as="p"
        className={cn(
          SCENE_ISLAND_WIDTH.line,
          "px-inset pb-inset-sm text-fg-secondary",
        )}
        variant="caption"
      >
        {t("editor-island-scene-hint")}
      </Text>
    ) : null,
    key: scene ? `scene-${scene.id}` : "scene",
    section:
      open === "layout" ? (
        <SceneLayoutGroup {...props} />
      ) : open === "autoZoom" ? (
        <SceneAutoZoomGroup
          hasAutoZooms={autoZoom?.hasAutoZooms ?? false}
          onClear={props.onClearAutoZooms}
          onRegenerate={props.onRegenerateAutoZooms}
        />
      ) : open === null ? null : (
        <ScenePaneGroup {...props} pane={open} />
      ),
    sectionKey: open ?? undefined,
    title: (
      <IslandRowTitle
        glyph={<SceneLayoutGlyph drawing={layoutDrawing(props)} />}
        title={t("editor-island-scene")}
      />
    ),
  };
}
