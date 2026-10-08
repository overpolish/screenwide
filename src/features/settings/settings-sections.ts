// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../i18n/i18n";

/** The panes the sidebar lists. */
export type SettingsSection =
  | "general"
  | "glide"
  | "ruler"
  | "annotate"
  | "ocr"
  | "moments"
  | "transcription"
  | "hotkeys";

/** The title the window wears for a pane, and the sidebar lists it under. */
export const sectionTitle = (section: SettingsSection) => {
  switch (section) {
    case "annotate":
      return t("settings-section-annotate");
    case "general":
      return t("settings-section-general");
    case "glide":
      return t("settings-section-glide");
    case "hotkeys":
      return t("settings-section-shortcuts");
    case "moments":
      return t("settings-section-moments");
    case "ocr":
      return t("settings-section-ocr");
    case "ruler":
      return t("settings-section-ruler");
    case "transcription":
      return t("settings-section-transcription");
  }
};
