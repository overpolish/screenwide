// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createContext, use } from "react";

import type { FrameFormatsControl } from "./use-recording-formats";

/** The recording's formats as the editor window holds them, for the parts
 * of the window that show and switch them: the timeline's format bar. Null
 * outside a recording. */
export const FrameFormatsContext = createContext<FrameFormatsControl | null>(
  null,
);

export const useFrameFormats = () => use(FrameFormatsContext);
