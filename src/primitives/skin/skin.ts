// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createContext, use } from "react";

// What a control is drawn on, which says how it is drawn: a window's
// material, or the island's black. The island sets it for all it holds, so
// a control there is the same component, with the same props, as in a
// window, drawn in the island's dress: a circle of the island's size for an
// icon, a pill for words, a fill that grows in under the pointer.

export type Skin = "island" | "window";

export const SkinContext = createContext<Skin>("window");

export const useSkin = () => use(SkinContext);
