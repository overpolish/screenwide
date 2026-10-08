// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";

export const isMacOS =
  typeof navigator !== "undefined" && navigator.userAgent.includes("Mac");

/**
 * A key's name in the app's language: the modifiers and named keys, written
 * as `hotkeyKeys` returns them or as a story spells them. Any other key, such
 * as a letter, is its own name.
 */
export const keyLabel = (key: string) => {
  switch (key.trim().toLowerCase()) {
    case "shift":
    case "⇧":
      return t("controls-key-shift");
    case "command":
    case "cmd":
    case "⌘":
      return t("controls-key-command");
    case "meta":
    case "super":
    case "win":
      return isMacOS ? t("controls-key-command") : t("controls-key-windows");
    case "control":
    case "ctrl":
    case "⌃":
      return isMacOS ? t("controls-key-control") : t("controls-key-ctrl");
    case "option":
    case "⌥":
      return t("controls-key-option");
    case "alt":
      return t("controls-key-alt");
    case "esc":
    case "escape":
      return t("controls-key-escape");
    case "space":
      return t("controls-key-space");
    case "enter":
    case "return":
      return t("controls-key-enter");
    case "tab":
      return t("controls-key-tab");
    case "delete":
      return t("controls-key-delete");
    case "backspace":
      return t("controls-key-backspace");
    case "mousemiddle":
      return t("controls-key-mouse-middle");
    case "mouseback":
      return t("controls-key-mouse-back");
    case "mouseforward":
      return t("controls-key-mouse-forward");
    default:
      return key;
  }
};
