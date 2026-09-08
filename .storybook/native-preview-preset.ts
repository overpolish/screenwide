// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fileURLToPath } from "node:url";

/**
 * Listed first in `addons` so the tool renders ahead of the theme switcher,
 * whose text-sized width would otherwise push it onto a subpixel.
 */
export const managerEntries = (entries: string[] = []) => [
  fileURLToPath(new URL("./native-preview-tool.tsx", import.meta.url)),
  ...entries,
];
