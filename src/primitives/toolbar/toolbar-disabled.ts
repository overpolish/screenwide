// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createContext } from "react";

// Disables every tool beneath it, for a bar out of reach as a whole; a chosen
// tool stays chosen and only stops answering. A window that gathers toolbars
// built elsewhere, as the Editor's titlebar does, provides it directly.
export const ToolbarDisabledContext = createContext(false);
