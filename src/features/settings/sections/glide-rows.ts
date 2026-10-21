// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { platformArg, t } from "../../../i18n/i18n";

// Glide's rows, shared with Settings search: its switches in Tools, and the
// keys held to steer it in Shortcuts.
export const glideToggles = () =>
  [
    {
      key: "cursorFollows",
      macOnly: false,
      title: t("settings-glide-cursor-follows"),
    },
    {
      description: t("settings-glide-haptics-description"),
      key: "haptics",
      macOnly: true,
      title: t("settings-glide-haptics"),
    },
    {
      description: t("settings-glide-double-tap-description"),
      key: "doubleTapCenter",
      macOnly: false,
      title: t("settings-glide-double-tap"),
    },
  ] as const;

export const modifierTitle = (key: ModifierKey) => {
  switch (key) {
    case "mouseModifier":
      return t("settings-glide-mouse-modifier");
    case "thirdsModifier":
      return t("settings-glide-thirds-modifier");
    case "spacesModifier":
      return t("settings-glide-spaces-modifier", { platform: platformArg() });
    case "monitorsModifier":
      return t("settings-glide-monitors-modifier");
  }
};

export const modifierDescription = (key: ModifierKey) => {
  switch (key) {
    case "mouseModifier":
      return t("settings-glide-mouse-modifier-description");
    case "thirdsModifier":
      return t("settings-glide-thirds-modifier-description");
    case "spacesModifier":
      return t("settings-glide-spaces-modifier-description", {
        platform: platformArg(),
      });
    case "monitorsModifier":
      return t("settings-glide-monitors-modifier-description");
  }
};

export const modifierKeys = [
  "mouseModifier",
  "thirdsModifier",
  "spacesModifier",
  "monitorsModifier",
] as const;

export type ModifierKey = (typeof modifierKeys)[number];
