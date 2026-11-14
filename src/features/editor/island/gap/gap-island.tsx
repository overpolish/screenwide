// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Trash2 } from "lucide-react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { BEAT_GLYPHS } from "../../glyphs/beat-glyphs";

import { GapSection } from "./gap-section";

import type { EditorIslandContent } from "../editor-island";
import type { BackgroundPartsProps } from "../frame/use-background-parts";

// A gap on the strip, in hand: time inserted into the recording with
// nothing recorded behind it, drawn as a blank scene of its own, showing its
// background and what is drawn over it. Its length is drawn on the strip, by
// its ends; its background is chosen from the Frame island's tiles, the
// recording's own until it has one, which Clear gap background goes back
// to; and taking it away, confirmed as every cut on the strip is, closes up
// the time it took.

/** What the gap sits on is `background`: its own, or the recording's while
 * it has none. */
export type GapIslandProps = BackgroundPartsProps & {
  /** Whether the gap has a background of its own. */
  hasOwnBackground: boolean;
  id: string;
  /** Takes the gap's own background away, so it shows the recording's. */
  onClearBackground: () => void;
  onRemove: () => void;
};

export function gapIsland(props: GapIslandProps): EditorIslandContent {
  const Glyph = BEAT_GLYPHS.gap;
  return {
    actions: (
      <ConfirmActionButton
        confirmLabel={t("editor-island-gap-confirm-remove")}
        icon={<Trash2 />}
        label={t("editor-island-gap-remove")}
        onConfirm={props.onRemove}
        variant="ghost"
      />
    ),
    // Hung from the row's end, so the library's column opens past its start,
    // as the Frame island's does (`IslandAside`).
    anchor: "end",
    key: `gap-${props.id}`,
    section: <GapSection {...props} />,
    title: (
      <IslandRowTitle
        glyph={<Glyph aria-hidden className="size-icon" />}
        title={t("editor-island-gap")}
      />
    ),
  };
}
