// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createContext } from "react";

export type ListBoxSize = "compact" | "default";

export const ListBoxSizeContext = createContext<ListBoxSize>("default");
