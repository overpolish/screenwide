// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  LayoutDashboard,
  type LucideIcon,
  Monitor,
  Trash2,
  Video,
  WandSparkles,
} from "lucide-react";
import { ToggleButtonGroup } from "react-aria-components";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";
import { IconToggleButton } from "../../../../primitives/button/icon-button";

import type { SceneGroup } from "./scene-framing";

// The end of a scene island's row: one choice of the group hanging open
// beneath, filled while it is, and, for a scene, deleting it, which asks
// twice. A group is always open, so pressing the open one leaves it open.

/** The groups of options the scene tool opens: a scene's framing, and with
 * no scene in hand the planned zooms too. */
export type SceneToolGroup = SceneGroup | "autoZoom";

const GROUP_GLYPHS: Record<SceneToolGroup, LucideIcon> = {
  autoZoom: WandSparkles,
  camera: Video,
  layout: LayoutDashboard,
  screen: Monitor,
};

const groupLabel = (group: SceneToolGroup) => {
  switch (group) {
    case "autoZoom":
      return t("editor-island-scene-auto-zoom");
    case "camera":
      return t("editor-island-scene-camera");
    case "layout":
      return t("editor-island-scene-layout");
    case "screen":
      return t("editor-island-scene-screen");
  }
};

export function SceneActions({
  groups,
  onDelete,
  onGroupChange,
  open,
}: {
  groups: SceneToolGroup[];
  onGroupChange: (group: SceneToolGroup) => void;
  open: SceneToolGroup | null;
  /** Deletes the scene; the recording's own framing has none. */
  onDelete?: () => void;
}) {
  return (
    <>
      {groups.length > 0 ? (
        <ToggleButtonGroup
          aria-label={t("editor-island-scene-groups")}
          className="flex items-center gap-separate"
          disallowEmptySelection
          onSelectionChange={(keys) => {
            const group = groups.find((each) => keys.has(each));
            if (group) onGroupChange(group);
          }}
          selectedKeys={open === null ? [] : [open]}
          selectionMode="single"
        >
          {groups.map((group) => {
            const Glyph: LucideIcon = GROUP_GLYPHS[group];
            return (
              <IconToggleButton
                aria-label={groupLabel(group)}
                id={group}
                key={group}
              >
                <Glyph aria-hidden className="size-icon" />
              </IconToggleButton>
            );
          })}
        </ToggleButtonGroup>
      ) : null}
      {onDelete ? (
        <ConfirmActionButton
          confirmLabel={t("editor-island-scene-confirm-delete")}
          icon={<Trash2 />}
          label={t("editor-island-scene-delete")}
          onConfirm={onDelete}
          variant="ghost"
        />
      ) : null}
    </>
  );
}
