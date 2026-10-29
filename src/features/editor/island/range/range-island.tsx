// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { MoveHorizontal, Scissors } from "lucide-react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";

import type { EditorIslandContent } from "../editor-island";

// A range drawn across the strip, in hand: its cut, confirmed as every cut
// on the strip is.

export type RangeIslandProps = {
  id: string;
  onCut: () => void;
};

export function rangeIsland({
  id,
  onCut,
}: RangeIslandProps): EditorIslandContent {
  return {
    actions: (
      <ConfirmActionButton
        confirmLabel={t("editor-island-range-confirm-cut")}
        icon={<Scissors />}
        label={t("editor-island-range-cut")}
        onConfirm={onCut}
        variant="ghost"
      />
    ),
    key: `range-${id}`,
    title: (
      <IslandRowTitle
        glyph={<MoveHorizontal aria-hidden className="size-icon" />}
        title={t("editor-island-range")}
      />
    ),
  };
}
