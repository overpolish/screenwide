// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { type IslandAnchor, type IslandNotch } from "./island-geometry";
import { type IslandPart } from "./island-outline-sync";

// What an island (`Island`) is given: its row, what hangs beneath it, and how
// the two sit and come and go.

export type IslandProps = {
  "aria-label": string;
  /** The controls along the top, usually an `IslandRow`. */
  row: ReactNode;
  /** Where under the row the section hangs. */
  anchor?: IslandAnchor;
  /** A row control to centre the section under, in place of `anchor`. */
  anchorControl?: HTMLElement | null;
  // Centred under the notch whatever the row does beside it, so a row growing
  // on one side widens the island on that side alone and nothing under it
  // moves (`island-stack-layout.ts`).
  /** A second line under the row, in its black; the section hangs below. */
  below?: ReactNode;
  /** Whether it is out; hidden, it is tucked into the notch, or gone. */
  isShown?: boolean;
  /** The screen's measured notch; left out on a screen without one. */
  notch?: IslandNotch | null;
  /** The island has tucked itself away after `isShown` turned false. */
  onHidden?: () => void;
  // Leaving is reported a moment after the pointer goes, so a slip does not
  // close what hovering opened.
  /** The pointer coming onto the row or section, or leaving them. */
  onHoverChange?: (isHovered: boolean) => void;
  /** The row's and section's places in the page, for click-through, and
   * from the island's own middle and top, for placing it. */
  onShapeChange?: (parts: IslandPart[], fromMiddle: IslandPart[]) => void;
  // So its parts keep to the island's ends rather than centring over the
  // black. Not under a notch, where the row keeps its gap under the notch.
  /** Stretches the row to a wider section's width. */
  rowFillsWidth?: boolean;
  /** What hangs beneath the row, if anything. */
  section?: ReactNode;
  /** Starts the section's content straight under the row, with no room of
   * its own above it. */
  sectionJoinsRow?: boolean;
  /** Makes each section at least the row's width, as one block. */
  sectionSpansRow?: boolean;
  // For an island on a picture, where a speck would read as something left
  // behind.
  /** Without a notch, shrinks past a speck to nothing. */
  shrinksToNothing?: boolean;
};
