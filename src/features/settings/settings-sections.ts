// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../i18n/i18n";

/** The panes the sidebar lists. */
export type SettingsSection =
  | "general"
  | "recording"
  | "tools"
  | "moments"
  | "transcription"
  | "hotkeys"
  | "licences";

/** The title the window wears for a pane, and the sidebar lists it under. */
export const sectionTitle = (section: SettingsSection) => {
  switch (section) {
    case "general":
      return t("settings-section-general");
    case "hotkeys":
      return t("settings-section-shortcuts");
    case "licences":
      return t("settings-section-licences");
    case "moments":
      return t("settings-section-moments");
    case "recording":
      return t("settings-section-recording");
    case "tools":
      return t("settings-section-tools");
    case "transcription":
      return t("settings-section-transcription");
  }
};
