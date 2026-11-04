// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Scissors, X } from "lucide-react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { appLocale, t } from "../../../../i18n/i18n";
import { IconButton } from "../../../../primitives/button/icon-button";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { RangeToolIcon } from "../../glyphs/range-tool-icon";

import type { EditorIslandContent } from "../editor-island";

// A range drawn across the strip with the range tool, in hand: how long it
// plays, its cut, confirmed as every cut on the strip is, and letting it
// go, for anyone who does not know ⌘D.

export type RangeIslandProps = {
  id: string;
  /** How long the range plays, in seconds. */
  length: number;
  onCut: () => void;
  onDeselect: () => void;
};

export function rangeIsland({
  id,
  length,
  onCut,
  onDeselect,
}: RangeIslandProps): EditorIslandContent {
  const seconds = new Intl.NumberFormat(appLocale().formatLocale, {
    maximumFractionDigits: 1,
    style: "unit",
    unit: "second",
    unitDisplay: "narrow",
  });
  return {
    actions: (
      <>
        <ConfirmActionButton
          confirmLabel={t("editor-island-range-confirm-cut")}
          icon={<Scissors />}
          label={t("editor-island-range-cut")}
          onConfirm={onCut}
          variant="ghost"
        />
        <IconButton
          aria-label={t("editor-island-range-deselect")}
          className="text-fg-secondary enabled:hover:text-fg"
          onPress={onDeselect}
        >
          <X aria-hidden />
        </IconButton>
      </>
    ),
    key: `range-${id}`,
    title: (
      <IslandRowTitle
        glyph={<RangeToolIcon aria-hidden className="size-icon" />}
        note={seconds.format(length)}
        title={t("editor-island-range")}
      />
    ),
  };
}
