// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { hotkeysEqual, parseHotkey } from "../../../lib/hotkey";

// Combinations a shortcut that works anywhere should not take: the system's
// own, which it keeps or loses to us, and the ones nearly every app uses,
// which a global shortcut would take from all of them. One list per
// platform, written as a recorded shortcut is (`Super` is ⌘ on a Mac and
// the Windows key on Windows), so they compare with the stored ones.

export type ReservedShortcut = {
  by: "apps" | "system";
  /** What the combination is used for there, in the app's language, read
   * after "for". */
  label: () => string;
};

type Listed = ReservedShortcut & { value: string };

const system = (value: string, label: () => string): Listed => ({
  by: "system",
  label,
  value,
});
const apps = (value: string, label: () => string): Listed => ({
  by: "apps",
  label,
  value,
});

const MAC: Listed[] = [
  system("Super+Space", () => t("settings-reserved-spotlight")),
  system("Control+Space", () => t("settings-reserved-input-source")),
  system("Control+Alt+Space", () => t("settings-reserved-input-source")),
  system("Super+Control+Space", () => t("settings-reserved-emoji")),
  system("Super+Tab", () => t("settings-reserved-switch-apps")),
  system("Super+Shift+Tab", () => t("settings-reserved-switch-apps")),
  system("Super+Backquote", () => t("settings-reserved-switch-windows")),
  system("Super+Shift+Digit3", () => t("settings-reserved-screenshot")),
  system("Super+Shift+Digit4", () => t("settings-reserved-screenshot")),
  system("Super+Shift+Digit5", () => t("settings-reserved-screenshot")),
  system("Super+Control+Shift+Digit3", () => t("settings-reserved-screenshot")),
  system("Super+Control+Shift+Digit4", () => t("settings-reserved-screenshot")),
  system("Super+Control+KeyQ", () => t("settings-reserved-lock")),
  system("Super+Shift+KeyQ", () => t("settings-reserved-log-out")),
  system("Super+Alt+Shift+KeyQ", () => t("settings-reserved-log-out")),
  system("Super+Alt+KeyD", () => t("settings-reserved-dock")),
  system("Control+ArrowUp", () => t("settings-reserved-mission-control")),
  system("Control+ArrowDown", () => t("settings-reserved-app-windows")),
  system("Control+ArrowLeft", () => t("settings-reserved-spaces")),
  system("Control+ArrowRight", () => t("settings-reserved-spaces")),
  system("Super+Alt+Escape", () => t("settings-reserved-force-quit")),
  apps("Super+KeyQ", () => t("settings-reserved-quit")),
  apps("Super+KeyW", () => t("settings-reserved-close")),
  apps("Super+KeyH", () => t("settings-reserved-hide")),
  apps("Super+Alt+KeyH", () => t("settings-reserved-hide-others")),
  apps("Super+KeyM", () => t("settings-reserved-minimize")),
  apps("Super+Control+KeyF", () => t("settings-reserved-full-screen")),
  apps("Super+Shift+Slash", () => t("settings-reserved-help")),
  apps("Super+KeyC", () => t("settings-reserved-copy")),
  apps("Super+KeyV", () => t("settings-reserved-paste")),
  apps("Super+KeyX", () => t("settings-reserved-cut")),
  apps("Super+KeyZ", () => t("settings-reserved-undo")),
  apps("Super+Shift+KeyZ", () => t("settings-reserved-redo")),
  apps("Super+KeyA", () => t("settings-reserved-select-all")),
  apps("Super+KeyS", () => t("settings-reserved-save")),
  apps("Super+KeyF", () => t("settings-reserved-find")),
  apps("Super+KeyN", () => t("settings-reserved-new")),
  apps("Super+KeyO", () => t("settings-reserved-open")),
  apps("Super+KeyP", () => t("settings-reserved-print")),
  apps("Super+KeyT", () => t("settings-reserved-new-tab")),
  apps("Control+Tab", () => t("settings-reserved-switch-tabs")),
  apps("Control+Shift+Tab", () => t("settings-reserved-switch-tabs")),
  apps("Super+Comma", () => t("settings-reserved-settings")),
];

const WINDOWS: Listed[] = [
  system("Alt+Tab", () => t("settings-reserved-switch-apps")),
  system("Alt+Shift+Tab", () => t("settings-reserved-switch-apps")),
  system("Alt+Space", () => t("settings-reserved-window-menu")),
  system("Alt+F4", () => t("settings-reserved-close-window")),
  system("PrintScreen", () => t("settings-reserved-screenshot")),
  system("Control+Escape", () => t("settings-reserved-start")),
  system("Control+Shift+Escape", () => t("settings-reserved-task-manager")),
  system("Control+Alt+Delete", () => t("settings-reserved-security")),
  apps("Control+KeyC", () => t("settings-reserved-copy")),
  apps("Control+KeyV", () => t("settings-reserved-paste")),
  apps("Control+KeyX", () => t("settings-reserved-cut")),
  apps("Control+KeyZ", () => t("settings-reserved-undo")),
  apps("Control+KeyY", () => t("settings-reserved-redo")),
  apps("Control+Shift+KeyZ", () => t("settings-reserved-redo")),
  apps("Control+KeyA", () => t("settings-reserved-select-all")),
  apps("Control+KeyS", () => t("settings-reserved-save")),
  apps("Control+KeyF", () => t("settings-reserved-find")),
  apps("Control+KeyN", () => t("settings-reserved-new")),
  apps("Control+KeyO", () => t("settings-reserved-open")),
  apps("Control+KeyP", () => t("settings-reserved-print")),
  apps("Control+KeyT", () => t("settings-reserved-new-tab")),
  apps("Control+KeyW", () => t("settings-reserved-close")),
  apps("Control+Tab", () => t("settings-reserved-switch-tabs")),
  apps("Control+Shift+Tab", () => t("settings-reserved-switch-tabs")),
];

// Windows keeps every combination with the Windows key for itself, and
// adds new ones with each release, so they are reserved as a rule.
const WINDOWS_KEY: ReservedShortcut = {
  by: "system",
  label: () => t("settings-reserved-windows-key"),
};

/** What the system or most apps already do with a combination, if
 * anything. */
export function reservedUse(
  value: string,
  isMac: boolean,
): ReservedShortcut | null {
  if (!isMac && parseHotkey(value, isMac)?.modifiers.includes("Super"))
    return WINDOWS_KEY;
  return (
    (isMac ? MAC : WINDOWS).find((listed) =>
      hotkeysEqual(listed.value, value, isMac),
    ) ?? null
  );
}
