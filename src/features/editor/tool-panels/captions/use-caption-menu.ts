// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { PopupPanelItem } from "../../../popup-panel/store";
import {
  PopupMenuAnchor,
  usePopupMenu,
} from "../../../popup-panel/use-popup-menu";

import { CaptionGap } from "./caption-text";

const MENU_PREFIX = "caption-words:";
const MENU_WIDTH = 200;

/** What a word of the caption text can be asked: edited or put back as it
 * was heard, a break in the gap before it placed or taken away, or hidden
 * or shown. */
export type CaptionMenuAction =
  | "clear-break"
  | "edit"
  | "hide"
  | "join"
  | "restore-spelling"
  | "show"
  | "split";

/** What the menu offers for a word: editing it, and putting back what was
 * heard where it was corrected; what can change in the gap before it, where
 * the two words were said one after the other; and whether it shows. */
export const captionMenuItems = ({
  gap,
  isCorrected,
  isShown,
}: {
  /** Null where no break can be placed before the word. */
  gap: CaptionGap | null;
  isCorrected: boolean;
  isShown: boolean;
}): (PopupPanelItem & { id: CaptionMenuAction })[] => [
  {
    icon: "edit" as const,
    id: "edit" as const,
    label: t("editor-panels-transcript-edit"),
    shortcut: "Enter",
  },
  ...(isCorrected
    ? [
        {
          icon: "spelling" as const,
          id: "restore-spelling" as const,
          label: t("editor-panels-transcript-restore-spelling"),
        },
      ]
    : []),
  ...(gap === "split"
    ? [
        {
          icon: "remove" as const,
          id: "clear-break" as const,
          label: t("editor-panels-captions-clear-split"),
        },
      ]
    : gap === "join"
      ? [
          {
            icon: "remove" as const,
            id: "clear-break" as const,
            label: t("editor-panels-captions-remove-join"),
          },
        ]
      : gap === "break"
        ? [
            {
              icon: "caption-join" as const,
              id: "join" as const,
              label: t("editor-panels-captions-join"),
            },
          ]
        : gap === "none"
          ? [
              {
                icon: "caption-split" as const,
                id: "split" as const,
                label: t("editor-panels-captions-split"),
              },
            ]
          : []),
  isShown
    ? {
        icon: "hide" as const,
        id: "hide" as const,
        label: t("editor-panels-captions-hide-word"),
        shortcut: "Backspace",
      }
    : {
        icon: "show" as const,
        id: "show" as const,
        label: t("editor-panels-captions-show-word"),
        shortcut: "Backspace",
      },
];

/** A right click on a word of the caption text, answered with the app's own
 * menu; the word travels with it, so a pick reaches the one it opened on. */
export function useCaptionMenu(
  onAction: (action: CaptionMenuAction, word: number) => void,
) {
  const openMenu = usePopupMenu({
    idPrefix: MENU_PREFIX,
    label: t("editor-panels-captions-actions"),
    mode: "menu",
    onSelect: (itemId, context) => {
      onAction(itemId as CaptionMenuAction, Number(context));
    },
    showsAllItems: true,
    width: MENU_WIDTH,
  });
  return (
    word: number,
    anchor: PopupMenuAnchor,
    items: readonly PopupPanelItem[],
  ) => openMenu({ anchor, context: word.toString(), items: [...items] });
}
