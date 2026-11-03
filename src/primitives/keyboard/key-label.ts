// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { platformArg, t } from "../../i18n/i18n";
import { isWindows } from "../../lib/platform";

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
      return isWindows()
        ? t("controls-key-windows")
        : t("controls-key-command");
    case "control":
    case "ctrl":
    case "⌃":
      return isWindows() ? t("controls-key-ctrl") : t("controls-key-control");
    case "option":
    case "⌥":
      return t("controls-key-option");
    case "alt":
      return t("controls-key-alt");
    case "esc":
    case "escape":
      return t("controls-key-escape", { platform: platformArg() });
    case "space":
      return t("controls-key-space");
    case "enter":
    case "return":
      return t("controls-key-enter");
    case "tab":
      return t("controls-key-tab", { platform: platformArg() });
    case "delete":
      return t("controls-key-delete");
    case "backspace":
      return t("controls-key-backspace");
    case "capslock":
      return t("controls-key-caps-lock");
    case "fn":
      return t("controls-key-fn");
    case "up":
      return t("controls-key-up");
    case "down":
      return t("controls-key-down");
    case "left":
      return t("controls-key-left");
    case "right":
      return t("controls-key-right");
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
