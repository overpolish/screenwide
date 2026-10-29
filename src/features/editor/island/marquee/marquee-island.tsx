// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SquareDashedMousePointer, Trash2 } from "lucide-react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";

import type { EditorIslandContent } from "../editor-island";

// Several annotations chosen at once with the marquee: how many, and
// deleting them all, which asks twice as every delete on the island does.
// They are of different kinds as often as not, so nothing of their look is
// offered; one alone is shown on its own pen's island instead.

export type MarqueeIslandProps = {
  /** How many annotations are chosen, two or more. */
  count: number;
  onDelete: () => void;
};

export function marqueeIsland({
  count,
  onDelete,
}: MarqueeIslandProps): EditorIslandContent {
  return {
    actions: (
      <ConfirmActionButton
        confirmLabel={t("editor-island-marquee-confirm-delete")}
        icon={<Trash2 aria-hidden />}
        label={t("editor-island-marquee-delete")}
        onConfirm={onDelete}
        variant="ghost"
      />
    ),
    key: "marquee",
    title: (
      <IslandRowTitle
        glyph={<SquareDashedMousePointer aria-hidden className="size-icon" />}
        title={t("editor-island-marquee", { count })}
      />
    ),
  };
}
