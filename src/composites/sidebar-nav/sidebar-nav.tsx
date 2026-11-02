// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ReactNode } from "react";

import { ChevronRight } from "lucide-react";
import { motion, useReducedMotion } from "motion/react";
import { useId, useState } from "react";
import { ToggleButton, ToggleButtonGroup } from "react-aria-components";

import { t } from "../../i18n/i18n";
import { motionDurations, motionEasings } from "../../lib/motion";
import { cn } from "../../lib/styling";
import { Button } from "../../primitives/button/button";
import { buttonVariants } from "../../primitives/button/button-variants";

type SidebarNavItem = {
  icon: ReactNode;
  id: string;
  label: string;
  isDisabled?: boolean;
  // As a source list keeps its Trash at the bottom; still one of the rows
  // for selection and the arrow keys.
  /** Sits at the foot of the list, apart from the rows above. */
  isPinned?: boolean;
};

export type SidebarNavProps = {
  "aria-label": string;
  items: SidebarNavItem[];
  onSelectionChange: (id: string) => void;
  selected: string;
  className?: string;
  defaultExpanded?: boolean;
  isDisabled?: boolean;
  /** Allow labels to expand and show the expand/collapse control. */
  isExpandable?: boolean;
  isExpanded?: boolean;
  onExpandedChange?: (expanded: boolean) => void;
  /** A right click on a row, at the pointer's place in the window. */
  onItemContextMenu?: (id: string, point: { x: number; y: number }) => void;
};

/** The widths the sidebar moves between: its labels shown, and only its
 * icons. Collapsed, it is a button's inset, an icon and the inset again, so
 * each icon stays where it sits in the open sidebar while the labels come
 * and go. */
const EXPANDED_WIDTH = "12rem";
const COLLAPSED_WIDTH = "calc(var(--spacing-inset) * 2 + var(--spacing-icon))";

export function SidebarNav({
  "aria-label": label,
  className,
  defaultExpanded = false,
  isDisabled,
  isExpandable = true,
  isExpanded,
  items,
  onExpandedChange,
  onItemContextMenu,
  onSelectionChange,
  selected,
}: SidebarNavProps) {
  const [localExpanded, setLocalExpanded] = useState(defaultExpanded);
  // A sidebar that cannot collapse can still be held open, the way System
  // Settings always shows its labels; only the toggle needs `isExpandable`.
  const expanded = isExpanded ?? (isExpandable && localExpanded);
  const groupId = useId();
  const reducedMotion = useReducedMotion();
  const transition = {
    duration: reducedMotion ? 0 : motionDurations.travel,
    ease: motionEasings.out,
  };
  const toggleLabel = expanded
    ? t("controls-sidebar-collapse")
    : t("controls-sidebar-expand");

  return (
    <motion.nav
      animate={{ width: expanded ? EXPANDED_WIDTH : COLLAPSED_WIDTH }}
      aria-label={label}
      className={cn(
        "flex min-h-0 shrink-0 flex-col gap-separate text-fg",
        className,
      )}
      initial={false}
      transition={transition}
    >
      <div
        className="flex min-h-0 flex-1 flex-col"
        id={groupId}
        onContextMenu={(event) => {
          // A disabled row can hand the click to its group rather than
          // taking it itself, so the row under the pointer is looked for
          // too: its menu is how an unavailable row is taken away.
          const row =
            (event.target as Element).closest<HTMLElement>("[data-item-id]") ??
            Array.from(
              event.currentTarget.querySelectorAll<HTMLElement>(
                "[data-item-id]",
              ),
            ).find((candidate) => {
              const bounds = candidate.getBoundingClientRect();
              return (
                event.clientY >= bounds.top && event.clientY <= bounds.bottom
              );
            });
          const id = row?.dataset.itemId;
          if (!onItemContextMenu || !id) return;
          event.preventDefault();
          onItemContextMenu(id, { x: event.clientX, y: event.clientY });
        }}
      >
        <ToggleButtonGroup
          aria-label={label}
          // Rows sit 2px apart so neighbouring fills stay apart.
          className="flex flex-1 flex-col gap-0.5"
          disallowEmptySelection
          isDisabled={isDisabled}
          onSelectionChange={(keys) => {
            const id = keys.values().next().value;
            if (typeof id === "string") onSelectionChange(id);
          }}
          orientation="vertical"
          selectedKeys={new Set([selected])}
          selectionMode="single"
        >
          {items.map((item) => (
            <ToggleButton
              aria-label={item.label}
              // A row is a ghost button with its content at the start, so its
              // inset, height, type, fills and focus ring are a button's and
              // it lines up with the buttons beside it. The chosen row is a
              // held toggle, filled in the primary colour as
              // `IconToggleButton` is, so where you are reads at a glance.
              className={({ isDisabled, isSelected }) =>
                cn(
                  buttonVariants(
                    isSelected
                      ? { align: "start", color: "primary", isDisabled }
                      : { align: "start", isDisabled, variant: "ghost" },
                  ),
                  "w-full overflow-hidden",
                  item.isPinned && "mt-auto",
                )
              }
              // React Aria takes `id` as the row's key and does not put it
              // on the button, so the context menu reads it from here.
              data-item-id={item.id}
              id={item.id}
              isDisabled={item.isDisabled}
              key={item.id}
            >
              {item.icon}
              <motion.span
                animate={{ opacity: expanded ? 1 : 0 }}
                aria-hidden
                className="flex min-w-0 flex-1 items-center"
                initial={false}
                transition={transition}
              >
                <span className="truncate">{item.label}</span>
              </motion.span>
            </ToggleButton>
          ))}
        </ToggleButtonGroup>
      </div>
      {isExpandable && (
        // A button rather than a square `IconButton`, so it is as wide as the
        // collapsed sidebar and its glyph stands on the rows' icon column.
        <Button
          aria-controls={groupId}
          aria-expanded={expanded}
          aria-label={toggleLabel}
          className="mt-auto self-start"
          onPress={() => {
            if (isExpanded === undefined) setLocalExpanded(!expanded);
            onExpandedChange?.(!expanded);
          }}
          variant="ghost"
        >
          <motion.span
            animate={{ rotate: expanded ? 180 : 0 }}
            className="flex"
            initial={false}
            transition={transition}
          >
            <ChevronRight className="transform-gpu rtl:-scale-x-100" />
          </motion.span>
        </Button>
      )}
    </motion.nav>
  );
}
