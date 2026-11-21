// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  LayoutDashboard,
  type LucideIcon,
  Trash2,
  WandSparkles,
} from "lucide-react";
import { ToggleButtonGroup } from "react-aria-components";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";
import { IconToggleButton } from "../../../../primitives/button/icon-button";
import { IslandParts } from "../../../../primitives/island/island-parts";

// The end of a scene island's row: one choice of the group hanging open
// beneath, filled while it is, where there is more than one, and, for a
// scene, deleting it, which asks twice. A group is always open, so pressing
// the open one leaves it open.

/** The groups of options the Scene mode opens: a scene's layout, and with
 * no scene in hand the planned zooms too. */
export type SceneToolGroup = "autoZoom" | "layout";

const GROUP_GLYPHS: Record<SceneToolGroup, LucideIcon> = {
  autoZoom: WandSparkles,
  layout: LayoutDashboard,
};

/** Every group, in the order the row shows those it has. */
const GROUPS: SceneToolGroup[] = ["layout", "autoZoom"];

const groupLabel = (group: SceneToolGroup) =>
  group === "autoZoom"
    ? t("editor-island-scene-auto-zoom")
    : t("editor-island-scene-layout");

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
      {groups.length > 1 ? (
        <ToggleButtonGroup
          aria-label={t("editor-island-scene-groups")}
          className="flex min-w-0 items-center"
          disallowEmptySelection
          onSelectionChange={(keys) => {
            const group = groups.find((each) => keys.has(each));
            if (group) onGroupChange(group);
          }}
          selectedKeys={open === null ? [] : [open]}
          selectionMode="single"
        >
          <IslandParts
            className="flex items-center gap-separate"
            direction="row"
          >
            {GROUPS.filter((group) => groups.includes(group)).map((group) => {
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
          </IslandParts>
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
