// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  getInteractionModality,
  setInteractionModality,
} from "react-aria/private/interactions/useFocusVisible";

/** Escape cancels interactions; it never starts keyboard navigation. Keep the
 * event available to controls and dismissal handlers, including when unused. */
export function preserveEscapeFocus() {
  let modality = getInteractionModality() ?? "pointer";
  const capture = (event: KeyboardEvent) => {
    if (event.key === "Escape")
      modality = getInteractionModality() ?? "pointer";
  };
  const restore = (event: KeyboardEvent) => {
    if (event.key === "Escape") setInteractionModality(modality);
  };
  for (const type of ["keydown", "keyup"] as const) {
    // Window capture precedes React Aria's document capture listener. This
    // document listener restores the snapshot before control handlers run.
    window.addEventListener(type, capture, { capture: true });
    document.addEventListener(type, restore, { capture: true });
  }
  return () => {
    for (const type of ["keydown", "keyup"] as const) {
      window.removeEventListener(type, capture, { capture: true });
      document.removeEventListener(type, restore, { capture: true });
    }
  };
}
