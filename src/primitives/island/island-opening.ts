// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

/** Whether the section is opening from closed, as against open and changing
 * or closing: the row's corner and the section's black are timed apart
 * while it opens. True from the render the section opens in until `finish`
 * is called, once its black has grown. */
export function useOpening(isOpen: boolean) {
  const [wasOpen, setWasOpen] = useState(isOpen);
  const [isOpening, setIsOpening] = useState(false);
  if (wasOpen !== isOpen) {
    setWasOpen(isOpen);
    setIsOpening(isOpen);
  }
  return {
    finish: () => {
      setIsOpening(false);
    },
    isOpening,
  };
}
