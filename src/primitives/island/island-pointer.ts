// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createContext } from "react";

/** Whether the pointer is on the island, as its window last found it; `null`
 * where nothing reports it, as in a story. The island's own pointer events
 * can miss the pointer leaving once its window lets the pointer through, so
 * this says it left. */
export const IslandPointerContext = createContext<boolean | null>(null);
