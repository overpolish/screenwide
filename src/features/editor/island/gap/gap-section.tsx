// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { Button } from "../../../../primitives/button/button";
import { IslandParts } from "../../../../primitives/island/island-parts";
import { FRAME_ISLAND_WIDTH } from "../frame/frame-width";
import { useBackgroundParts } from "../frame/use-background-parts";

import type { GapIslandProps } from "./gap-island";

/** Under the gap island's row: its background's tiles and colours, and
 * clearing its own background; its length is drawn on the strip, by its
 * ends. The island is never narrower than the Frame island, so the tiles
 * wrap the same way in both, and the library's pictures open in a column
 * beside it. */
export function GapSection(props: GapIslandProps) {
  const parts = useBackgroundParts(props);
  return (
    // The tiles open the section, whose box clips what reaches past its
    // top, so a chosen tile's outline in the first row needs the small
    // inset above it; in the Frame island the size line gives it that room.
    <div className="flex">
      <IslandParts
        className={cn(
          "flex shrink-0 flex-col gap-separate self-start pt-inset-sm",
          FRAME_ISLAND_WIDTH.section,
        )}
      >
        {parts.tiles}
        {parts.colors}
        <div className="flex justify-end gap-separate" key="actions">
          <Button
            isDisabled={!props.hasOwnBackground}
            onPress={props.onClearBackground}
          >
            {t("editor-island-gap-clear-background")}
          </Button>
        </div>
      </IslandParts>
      {parts.column}
    </div>
  );
}
