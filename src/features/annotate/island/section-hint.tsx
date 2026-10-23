// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { Text } from "../../../primitives/text/text";

/** A note at the end of a pen, after its circles. The circles' glyphs sit
 * the small inset in from their edges, so the note ends that much short of
 * the section's edge too: the black then frames glyph and text alike, and
 * the section reads as evenly inset at both ends. */
export function SectionHint({ children }: { children: ReactNode }) {
  return (
    <Text className="max-w-48 pe-inset-sm text-fg-secondary" variant="caption">
      {children}
    </Text>
  );
}
