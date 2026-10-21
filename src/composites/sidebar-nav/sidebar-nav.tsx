// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ReactNode } from "react";

import { ChevronRight } from "lucide-react";
import { motion, useReducedMotion } from "motion/react";
import { useId, useState } from "react";
import { ToggleButton, ToggleButtonGroup } from "react-aria-components";

import { t } from "../../i18n/i18n";
import {
  motionDurationCss,
  motionDurations,
  motionEasings,
} from "../../lib/motion";
import { cn, elementFocusVisible, focusStyles } from "../../lib/styling";
import { IconButton } from "../../primitives/button/icon-button";

type SidebarNavItem = {
  icon: ReactNode;
  id: string;
  label: string;
  isDisabled?: boolean;
  /** Sits at the foot of the list, apart from the rows above it, the way a
   * source list keeps its Trash at the bottom. Still one of the same rows
   * for selection and the arrow keys. */
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
  /** A right click on a row, with the row's id and the pointer's position in
   * the window. */
  onItemContextMenu?: (id: string, point: { x: number; y: number }) => void;
};

/** The widths the sidebar moves between: its labels shown, and only its
 * icons, where it is a control's height across so each row is a square. */
const EXPANDED_WIDTH = "12rem";
const COLLAPSED_WIDTH = "1.75rem";

// A row reads as a button: a control's height, the label type, a 16px icon.
// Its inset is the tight one so that collapsed, at a control's height across,
// the icon sits centred in a square. Rows sit 2px apart so neighbouring
// highlights stay apart. A row comes up on the fill under the pointer and
// deeper while pressed; the chosen one is filled in the primary colour, so
// where you are reads at a glance.
const rowClassName = [
  "group relative flex h-control-height w-full cursor-default items-center gap-inline overflow-hidden rounded-control px-inset-sm text-start text-label text-fg outline-none transition-colors",
  "data-[hovered]:not-data-[selected]:bg-fill data-[pressed]:not-data-[selected]:bg-fill-strong",
  "data-[selected]:bg-primary data-[selected]:text-on-primary data-[selected]:primary-raised",
  "data-[disabled]:opacity-40",
  focusStyles,
  elementFocusVisible,
];

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
              className={cn(rowClassName, item.isPinned && "mt-auto")}
              // React Aria takes `id` as the row's key and does not put it
              // on the button, so the context menu reads it from here.
              data-item-id={item.id}
              id={item.id}
              isDisabled={item.isDisabled}
              key={item.id}
              style={{
                transitionDuration: reducedMotion
                  ? "0s"
                  : motionDurationCss("state"),
              }}
            >
              <span
                aria-hidden
                className="flex size-icon shrink-0 transform-gpu items-center justify-center [&_svg]:size-icon [&_svg]:transform-gpu"
              >
                {item.icon}
              </span>
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
        <IconButton
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
        </IconButton>
      )}
    </motion.nav>
  );
}
