// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PopupPanelItem } from "../../popup-panel/store";
import { boundsAnchor, usePopupMenu } from "../../popup-panel/use-popup-menu";

/** One menu per saved background, so a selection names the tile it came
 * from without the panel having to carry it back. */
const MENU_PREFIX = "background-preset:";
/** The one action reads wider than the swatch it hangs off. */
const MENU_WIDTH = 180;

const items: PopupPanelItem[] = [
  { icon: "trash", id: "remove", label: "Remove Preset" },
];

/**
 * A right click on a saved background, answered with the app's own menu.
 *
 * The menu opens in the shared listbox window with the tool panel as its
 * parent, so it floats over the panel rather than inside it. An outside press
 * dismisses the menu alone: a tool panel is sticky, and stays where it is.
 */
export function useBackgroundPresetMenu(onRemove: (presetId: string) => void) {
  const openMenu = usePopupMenu({
    idPrefix: MENU_PREFIX,
    label: "Background actions",
    mode: "menu",
    onSelect: (itemId, presetId) => {
      if (itemId === "remove") onRemove(presetId);
    },
    width: MENU_WIDTH,
  });

  return (presetId: string, anchor: DOMRect) =>
    openMenu({ anchor: boundsAnchor(anchor), context: presetId, items });
}
