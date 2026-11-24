// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { Alert } from "../../../../primitives/alert/alert";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { Text } from "../../../../primitives/text/text";
import { SceneLayoutGlyph } from "../../glyphs/scene-layout-glyph";
import { FramingMenu } from "../framing-menu";

import { SceneActions, type SceneToolGroup } from "./scene-actions";
import { SceneAutoZoomGroup } from "./scene-auto-zoom-group";
import { layoutDrawing, type SceneFramingProps } from "./scene-framing";
import { SceneLayoutGroup } from "./scene-layout-group";
import { ScenePaneToggles } from "./scene-pane-toggles";

import type { ToolPanelFormats } from "../../tools/tool-panel-frame";
import type { FramingPatch } from "../../tools/tool-panel-patch";
import type { EditorIslandContent } from "../editor-island";

// The Scene mode: how the picture is framed. With a scene in hand it frames
// that scene's stretch, and can delete it; with none, it frames the
// recording wherever no scene covers it, and holds the zooms planned from
// the cursor and keys. The row, drawn as the layout, chooses which group of
// options hangs beneath where there is more than one; the one last chosen
// opens again, or the layout where that group is missing. Nothing here adds
// a scene: one is drawn along the strip's scene line and set up once in
// hand, which the row's note says while none is. The row also ends with
// whether the scene shows each pane, as glyphs (`ScenePaneToggles`), and
// the layout group holds the layouts; moving, zooming and rounding a pane
// is done on the picture (Select moves and resizes a box, the pane's crop
// picks what it shows). A pane's corners, and whether the camera
// follows the face, belong to its layer and are set on the Screen and
// Camera islands.

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
  /** A recording's formats, and their asks of the platform in hand's
   * framing, which a scene laid out here is part of. */
  framing?: {
    formats: ToolPanelFormats;
    onPatch: (patch: FramingPatch) => void;
  };
};

export function sceneIsland(props: SceneIslandProps): EditorIslandContent {
  const { autoZoom, scene } = props;
  const groups: SceneToolGroup[] = [
    "layout",
    ...(scene === null && autoZoom ? (["autoZoom"] as const) : []),
  ];
  const open: SceneToolGroup | null = groups.includes(props.group)
    ? props.group
    : groups.length > 0
      ? groups[0]
      : null;
  return {
    actions: (
      <>
        {props.framing && scene ? (
          <FramingMenu
            formats={props.framing.formats}
            onPatch={props.framing.onPatch}
          />
        ) : null}
        {props.panesShown ? <ScenePaneToggles {...props} /> : null}
        <SceneActions
          groups={groups}
          onDelete={scene ? props.onDelete : undefined}
          onGroupChange={props.onGroupChange}
          open={open}
        />
      </>
    ),
    anchor: "end",
    key: scene ? `scene-${scene.id}` : "scene",
    // The open group, then at the island's foot what a note has to say: a
    // paused scene says so as the island's alerts do, centred; with no scene
    // in hand, a note on the island's text column says how to add one. Each
    // takes the width the group sets, so its words wrap to it.
    section: (
      <div className="flex flex-col gap-separate">
        {open === "layout" ? (
          <SceneLayoutGroup {...props} />
        ) : open === "autoZoom" ? (
          <SceneAutoZoomGroup
            hasAutoZooms={autoZoom?.hasAutoZooms ?? false}
            onClear={props.onClearAutoZooms}
            onRegenerate={props.onRegenerateAutoZooms}
          />
        ) : null}
        {scene?.isPaused ? (
          <Alert className="w-0 min-w-full" role="status">
            {t("editor-island-scene-paused")}
          </Alert>
        ) : scene === null ? (
          <Text
            as="p"
            className="w-0 min-w-full px-inset-sm text-fg-secondary"
            variant="caption"
          >
            {t("editor-island-scene-hint")}
          </Text>
        ) : null}
      </div>
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
