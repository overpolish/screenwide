// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PopupPanelItem } from "../../popup-panel/store";
import { boundsAnchor, usePopupMenu } from "../../popup-panel/use-popup-menu";

/** One menu per saved colour, so a selection names the tile it came from
 * without the panel having to carry it back. */
const MENU_PREFIX = "annotation-color:";
/** The one action reads wider than the swatch it hangs off. */
const MENU_WIDTH = 180;

const items: PopupPanelItem[] = [
  { icon: "trash", id: "remove", label: "Remove Colour" },
];

/**
 * A right click on a colour of your own, answered with the app's own menu.
 *
 * The twin of `use-background-preset-menu.ts`: a saved colour is forgotten
 * the same way a saved background is, so the two rows of swatches behave
 * alike wherever they are met.
 */
export function useAnnotationColorMenu(onRemove: (color: string) => void) {
  const openMenu = usePopupMenu({
    idPrefix: MENU_PREFIX,
    label: "Colour actions",
    mode: "menu",
    onSelect: (itemId, color) => {
      if (itemId === "remove") onRemove(color);
    },
    width: MENU_WIDTH,
  });

  return (color: string, anchor: DOMRect) =>
    openMenu({ anchor: boundsAnchor(anchor), context: color, items });
}
