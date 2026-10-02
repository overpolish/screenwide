// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PopupPanelItem } from "../../../popup-panel/store";
import {
  boundsAnchor,
  usePopupMenu,
} from "../../../popup-panel/use-popup-menu";

/** One menu per template, so a selection names the tile it came from
 * without the panel having to carry it back. */
const MENU_PREFIX = "scene-template:";
/** The one action reads wider than the tile's corner it hangs off. */
const MENU_WIDTH = 180;

const items: PopupPanelItem[] = [
  { icon: "trash", id: "remove", label: "Remove Template" },
];

/**
 * A right click on a scene template, answered with the app's own menu. The
 * twin of `use-annotation-color-menu.ts`: a template is forgotten the way a
 * saved colour or background is.
 */
export function useSceneTemplateMenu(onRemove: (templateId: string) => void) {
  const openMenu = usePopupMenu({
    idPrefix: MENU_PREFIX,
    label: "Template actions",
    mode: "menu",
    onSelect: (itemId, templateId) => {
      if (itemId === "remove") onRemove(templateId);
    },
    width: MENU_WIDTH,
  });

  return (templateId: string, anchor: DOMRect) =>
    openMenu({ anchor: boundsAnchor(anchor), context: templateId, items });
}
