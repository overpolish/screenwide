// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingVideoTrackId } from "../types";

import {
  LayerContextMenu,
  LayerContextMenuState,
} from "./screenshot-layer-context-menu";

export function RecordingTrackContextMenu({
  menu,
  onClose,
  onMove,
}: {
  menu: LayerContextMenuState<RecordingVideoTrackId> | null;
  onClose: () => void;
  onMove: (
    track: RecordingVideoTrackId,
    direction: "backward" | "forward",
  ) => void;
}) {
  if (!menu) return null;
  return (
    <LayerContextMenu
      ariaLabel="Video layer actions"
      canDelete={false}
      menu={menu}
      onClose={onClose}
      onDelete={() => undefined}
      onMoveBackward={() => {
        onMove(menu.itemId, "backward");
      }}
      onMoveForward={() => {
        onMove(menu.itemId, "forward");
      }}
      showDelete={false}
    />
  );
}
