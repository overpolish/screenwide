// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { FEATURES } from "../../../../lib/feature-flags";
import {
  PopupMenuAnchor,
  usePopupMenu,
} from "../../../popup-panel/use-popup-menu";
import {
  exportCaptionTemplate,
  importCaptionTemplate,
  removeCaptionTemplate,
} from "../../recording/captions/captions-api";

import type { CaptionTemplate } from "../../../../bindings/CaptionTemplate";
import type { CaptionTemplateId } from "../../../../bindings/CaptionTemplateId";
import type { CaptionTemplateSource } from "../../../../bindings/CaptionTemplateSource";
import type { PopupPanelItem } from "../../../popup-panel/store";

const MENU_PREFIX = "caption-template:";
const MENU_WIDTH = 220;
const IMPORT = "import";
const EXPORT = "export";
const REMOVE = "remove";
const RESET_TO_ORIGINAL = "reset-to-original";

/**
 * The chosen template's menu, in the app's own pop-up: import one someone
 * shared, export this one as a zip, remove it where it is one of yours, or
 * put it back to how it ships. The template files' actions show only where
 * `FEATURES.captionTemplateFiles` is on. The template travels with the
 * menu, so a pick reaches the one it opened on. An imported template is
 * handed to `onImported`, to be taken up.
 */
export function useCaptionTemplateActions({
  onImported,
  onResetToOriginal,
  sources,
}: {
  onImported: (template: CaptionTemplate) => void;
  onResetToOriginal: (template: CaptionTemplateId) => void;
  sources: Partial<Record<CaptionTemplateId, CaptionTemplateSource>>;
}) {
  const openMenu = usePopupMenu({
    idPrefix: MENU_PREFIX,
    label: t("editor-panels-template-actions"),
    mode: "menu",
    onSelect: (itemId, template) => {
      // Any failure is the app's to report, in an alert of its own.
      const reported = (cause: unknown) => {
        console.error("Could not change the caption templates", cause);
      };
      if (itemId === IMPORT)
        importCaptionTemplate()
          .then((imported) => {
            if (imported) onImported(imported);
          })
          .catch(reported);
      else if (itemId === EXPORT)
        exportCaptionTemplate(template).catch(reported);
      else if (itemId === REMOVE)
        removeCaptionTemplate(template).catch(reported);
      else if (itemId === RESET_TO_ORIGINAL) onResetToOriginal(template);
    },
    showsAllItems: true,
    width: MENU_WIDTH,
  });
  return (template: CaptionTemplateId, anchor: PopupMenuAnchor) => {
    const files: PopupPanelItem[] = [
      {
        icon: "import",
        id: IMPORT,
        label: t("editor-panels-captions-template-import"),
      },
      {
        icon: "export",
        id: EXPORT,
        label: t("editor-panels-captions-template-export"),
      },
      ...(sources[template] === "user"
        ? [
            {
              icon: "trash" as const,
              id: REMOVE,
              label: t("editor-panels-captions-template-remove"),
            },
          ]
        : []),
    ];
    const items: PopupPanelItem[] = [
      ...(FEATURES.captionTemplateFiles ? files : []),
      {
        icon: "restore",
        id: RESET_TO_ORIGINAL,
        label: t("editor-panels-captions-reset-original"),
      },
    ];
    return openMenu({ anchor, context: template, items });
  };
}
