// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** The panes the sidebar lists, and the title the window wears for each. */
export type SettingsSection = "general" | "glide" | "ruler" | "ocr" | "hotkeys";

export const sectionTitles: Record<SettingsSection, string> = {
  general: "General",
  glide: "Glide",
  hotkeys: "Shortcuts",
  ocr: "OCR",
  ruler: "Ruler",
};
