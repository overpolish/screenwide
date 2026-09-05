// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ReactNode } from "react";

import { ChevronRight } from "lucide-react";
import { motion, useReducedMotion } from "motion/react";
import { useId, useState } from "react";
import {
  ToggleButton,
  ToggleButtonGroup,
  TooltipTrigger,
} from "react-aria-components";

import {
  motionDurationCss,
  motionDurations,
  motionEasings,
} from "../../../lib/motion";
import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";
import { IconButton } from "../button/icon-button";
import { defaultIconControlStyles } from "../button/icon-button-variants";
import { Tooltip } from "../tooltip/tooltip";

export type SidebarNavItem = {
  icon: ReactNode;
  id: string;
  label: string;
  isDisabled?: boolean;
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
};

export function SidebarNav({
  "aria-label": label,
  className,
  defaultExpanded = false,
  isDisabled,
  isExpandable = true,
  isExpanded,
  items,
  onExpandedChange,
  onSelectionChange,
  selected,
}: SidebarNavProps) {
  const [localExpanded, setLocalExpanded] = useState(defaultExpanded);
  const expanded = isExpandable && (isExpanded ?? localExpanded);
  const groupId = useId();
  const reducedMotion = useReducedMotion();
  const transition = {
    duration: reducedMotion ? 0 : motionDurations.travel,
    ease: motionEasings.out,
  };
  const toggleLabel = expanded ? "Collapse sidebar" : "Expand sidebar";

  return (
    <motion.nav
      animate={{ width: expanded ? "12rem" : "2.25rem" }}
      aria-label={label}
      className={cn(
        "gap-section flex min-h-0 shrink-0 flex-col text-content-fg",
        className,
      )}
      initial={false}
      transition={transition}
    >
      <div id={groupId}>
        <ToggleButtonGroup
          aria-label={label}
          className="gap-tight flex flex-col"
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
            <TooltipTrigger isDisabled={expanded} key={item.id}>
              <ToggleButton
                aria-label={item.label}
                className={cn(
                  defaultIconControlStyles,
                  "gap-control-inset flex w-full items-center overflow-hidden text-left text-sm font-semibold text-muted outline-none transition-colors",
                  "data-[hovered]:bg-neutral data-[hovered]:text-content-fg data-[pressed]:bg-neutral-hover",
                  "data-[selected]:bg-neutral data-[selected]:text-content-fg",
                  "data-[disabled]:cursor-not-allowed! data-[disabled]:text-neutral-disabled-fg! data-[disabled]:data-[selected]:bg-neutral-subtle",
                  focusStyles,
                  elementFocusVisible,
                )}
                id={item.id}
                isDisabled={item.isDisabled}
                style={{
                  transitionDuration: reducedMotion
                    ? "0s"
                    : motionDurationCss("state"),
                }}
              >
                <span
                  aria-hidden
                  className="flex size-5 shrink-0 transform-gpu items-center justify-center"
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
              <Tooltip placement="right">{item.label}</Tooltip>
            </TooltipTrigger>
          ))}
        </ToggleButtonGroup>
      </div>
      {isExpandable && (
        <TooltipTrigger>
          <IconButton
            aria-controls={groupId}
            aria-expanded={expanded}
            aria-label={toggleLabel}
            className="mt-auto self-start"
            onPress={() => {
              if (isExpanded === undefined) setLocalExpanded(!expanded);
              onExpandedChange?.(!expanded);
            }}
          >
            <motion.span
              animate={{ rotate: expanded ? 180 : 0 }}
              className="flex"
              initial={false}
              transition={transition}
            >
              <ChevronRight className="transform-gpu" />
            </motion.span>
          </IconButton>
          <Tooltip placement="right">{toggleLabel}</Tooltip>
        </TooltipTrigger>
      )}
    </motion.nav>
  );
}
