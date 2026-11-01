// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** A colour grid: the palette, `savedColors`, and Custom. */
export type PopupPanelColorsContent = {
  kind: "colors";
  savedColors: string[];
  value: string;
};
