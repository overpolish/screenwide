// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** What the frame panel shows about the output canvas. */
export type ToolPanelFrame = {
  height: number;
  sourceHeight: number;
  sourceWidth: number;
  width: number;
  /** How rounded the output canvas corners are, as a share of its shorter side. */
  radius?: number;
};
