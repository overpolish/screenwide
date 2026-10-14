// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";

import {
  addCaptionTemplateFonts,
  followCaptionTemplates,
} from "./captions-api";

const add = () => {
  addCaptionTemplateFonts().catch((cause: unknown) => {
    console.error("Could not add the caption templates' fonts", cause);
  });
};

/** Adds the fonts your templates bring to this window's, and those of any
 * template added later, so each family is named in its own type wherever
 * the window lists it. */
export function useCaptionTemplateFonts() {
  useEffect(() => {
    add();
    return followCaptionTemplates(add);
  }, []);
}
