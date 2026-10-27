// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type LucideIcon } from "lucide-react";
import { type ReactNode } from "react";

import { IslandButton } from "../../../../primitives/island/island-controls";

import { type IslandContent } from "./island-parts";
import { VisibilityButton } from "./visibility-button";

// An overlay drawn over the whole recording, such as the captions or the
// shortcuts, as the tools lay out their options: each group of choices an
// icon in the row, which opens the group under it and closes it again, and
// at the row's end an eye that shows or hides the overlay.

/** Which part of an island is open under its row, by key, so choosing
 * something else closes it. */
export type Expanded = {
  isOpen: (key: string) => boolean;
  toggle: (key: string) => void;
};

export type OverlayPart = {
  icon: LucideIcon;
  id: string;
  label: string;
  node: ReactNode;
};

export function overlayParts({
  expanded,
  isShown,
  key,
  onToggleShown,
  parts,
  showLabel,
}: {
  expanded: Expanded;
  isShown: boolean;
  key: string;
  onToggleShown: () => void;
  parts: OverlayPart[];
  showLabel: string;
}): Pick<IslandContent, "actions" | "section"> {
  const partKey = (part: OverlayPart) => `${key}-${part.id}`;
  const open = parts.find((part) => expanded.isOpen(partKey(part)));
  return {
    actions: (
      <>
        {parts.map((part) => {
          const isOpen = open === part;
          return (
            <IslandButton
              aria-expanded={isOpen}
              aria-label={part.label}
              isOpen={isOpen}
              key={part.id}
              onPress={() => {
                expanded.toggle(partKey(part));
              }}
            >
              <part.icon aria-hidden className="size-icon" />
            </IslandButton>
          );
        })}
        <VisibilityButton
          isShown={isShown}
          label={showLabel}
          onToggle={onToggleShown}
        />
      </>
    ),
    section: open ? { key: partKey(open), node: open.node } : undefined,
  };
}
