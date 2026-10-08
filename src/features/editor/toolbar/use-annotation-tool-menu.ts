// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { annotationToolLabel } from "../../../components/shared/annotation-style/annotation-text";
import { t } from "../../../i18n/i18n";
import {
  boundsAnchor,
  closePopupMenu,
  usePopupMenu,
} from "../../popup-panel/use-popup-menu";
import { ANNOTATION_TOOLS, AnnotationTool } from "../tool-panels/tool-registry";

const MENU_PREFIX = "annotation-tools:";
/** The longest label and its shortcut key, with room to spare. */
const MENU_WIDTH = 180;

/** Puts the menu away, if it is showing. */
export const closeAnnotationToolMenu = () => closePopupMenu(MENU_PREFIX);

/**
 * The annotation tools a narrow title bar has no room for, offered in the
 * app's own menu window: it floats over the bar and is clamped to the
 * monitor, so it fits however small the editor window is. It is shown whole,
 * never scrolled: it is the toolbar's own tail, read at a glance.
 */
export function useAnnotationToolMenu(
  onChoose: (tool: AnnotationTool) => void,
) {
  const openMenu = usePopupMenu({
    idPrefix: MENU_PREFIX,
    label: t("editor-toolbar-more-annotation-tools"),
    mode: "menu",
    onSelect: (itemId) => {
      const tool = ANNOTATION_TOOLS.find((item) => item.id === itemId);
      if (tool) onChoose(tool);
    },
    showsAllItems: true,
    width: MENU_WIDTH,
  });

  return (tools: readonly AnnotationTool[], anchor: DOMRect) =>
    openMenu({
      anchor: boundsAnchor(anchor),
      items: tools.map(({ id, shortcut }) => ({
        icon: `tool-${id}` as const,
        id,
        label: annotationToolLabel(id),
        shortcut,
      })),
    });
}
