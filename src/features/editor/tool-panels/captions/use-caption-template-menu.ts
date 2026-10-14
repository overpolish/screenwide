// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import {
  PopupMenuAnchor,
  usePopupMenu,
} from "../../../popup-panel/use-popup-menu";

import type { CaptionTemplateId } from "../../../../bindings/CaptionTemplateId";

const MENU_PREFIX = "caption-template:";
const MENU_WIDTH = 200;
const RESET_TO_ORIGINAL = "reset-to-original";

/** A right click on a caption template's tile, answered with the app's own
 * menu: putting that template back to how it ships. The template travels
 * with it, so a pick reaches the one it opened on. */
export function useCaptionTemplateMenu(
  onResetToOriginal: (template: CaptionTemplateId) => void,
) {
  const openMenu = usePopupMenu({
    idPrefix: MENU_PREFIX,
    label: t("editor-panels-template-actions"),
    mode: "menu",
    onSelect: (itemId, context) => {
      if (itemId === RESET_TO_ORIGINAL) onResetToOriginal(context);
    },
    showsAllItems: true,
    width: MENU_WIDTH,
  });
  return (template: CaptionTemplateId, anchor: PopupMenuAnchor) =>
    openMenu({
      anchor,
      context: template,
      items: [
        {
          icon: "restore",
          id: RESET_TO_ORIGINAL,
          label: t("editor-panels-captions-reset-original"),
        },
      ],
    });
}
