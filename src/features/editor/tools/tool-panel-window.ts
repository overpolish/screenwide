// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { EditorKind } from "../types";

/**
 * The panel window a workspace's tools open in.
 *
 * Each editor has one of its own, so a tool in hand in the recording window
 * and one in the screenshot window are both on screen at once, and neither
 * can take the other's panel away.
 */
export const toolPanelLabel = (workspace: EditorKind) =>
  `tool-panel-${workspace}`;
