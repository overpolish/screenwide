// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createContext } from "react";

/**
 * Disables every `ToolToggle` beneath it, for a toolbar that is out of reach
 * as a whole. The tools that build the bar need not know why: a toggle stays
 * selected as it was, and only stops answering.
 */
export const ToolToggleDisabledContext = createContext(false);
