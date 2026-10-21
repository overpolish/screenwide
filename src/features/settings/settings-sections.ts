// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../i18n/i18n";

/** The panes the sidebar lists. */
export type SettingsSection =
  "general" | "tools" | "moments" | "transcription" | "hotkeys";

/** The title the window wears for a pane, and the sidebar lists it under. */
export const sectionTitle = (section: SettingsSection) => {
  switch (section) {
    case "general":
      return t("settings-section-general");
    case "hotkeys":
      return t("settings-section-shortcuts");
    case "moments":
      return t("settings-section-moments");
    case "tools":
      return t("settings-section-tools");
    case "transcription":
      return t("settings-section-transcription");
  }
};
