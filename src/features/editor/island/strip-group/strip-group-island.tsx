// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { GalleryHorizontal, Scissors } from "lucide-react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";

import type { EditorIslandContent } from "../editor-island";

// Several beats and lulls gathered on the strip with Shift: how many, and
// cutting them all with their time, confirmed as every cut on the strip
// is. Beats and lulls mixed have nothing else in common to offer, as the
// Marquee island's annotations have none; one alone shows its own island.

export type StripGroupIslandProps = {
  /** How many beats and lulls are chosen, two or more. */
  count: number;
  /** Names the choice, so a new one is a new island. */
  id: string;
  onCut: () => void;
};

export function stripGroupIsland({
  count,
  id,
  onCut,
}: StripGroupIslandProps): EditorIslandContent {
  return {
    actions: (
      <ConfirmActionButton
        confirmLabel={t("editor-island-strip-group-confirm-cut")}
        icon={<Scissors />}
        label={t("editor-island-strip-group-cut")}
        onConfirm={onCut}
        variant="ghost"
      />
    ),
    key: `strip-group-${id}`,
    title: (
      <IslandRowTitle
        glyph={<GalleryHorizontal aria-hidden className="size-icon" />}
        title={t("editor-island-strip-group", { count })}
      />
    ),
  };
}
