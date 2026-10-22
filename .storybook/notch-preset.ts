// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fileURLToPath } from "node:url";

/** Registers the notch switch, which shows only on stories that use it. */
export const managerEntries = (entries: string[] = []) => [
  ...entries,
  fileURLToPath(new URL("./notch-tool.tsx", import.meta.url)),
];
