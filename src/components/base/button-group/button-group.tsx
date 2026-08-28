// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CSSProperties, KeyboardEvent } from "react";
import {
  Toolbar as AriaToolbar,
  ToolbarProps as AriaToolbarProps,
} from "react-aria-components";

import { cn } from "../../../lib/styling";

import {
  getNextGridItemIndex,
  GridNavigationDirection,
} from "./button-grid-navigation";

type ButtonGroupProps = Omit<AriaToolbarProps, "className"> & {
  className?: string;
};

export function ButtonGroup({
  className,
  orientation = "horizontal",
  ...props
}: ButtonGroupProps) {
  return (
    <AriaToolbar
      {...props}
      className={cn(
        "flex",
        orientation === "horizontal" ? "flex-row" : "flex-col",
        className,
      )}
      orientation={orientation}
    />
  );
}

type ButtonGridProps = Omit<
  AriaToolbarProps,
  "className" | "onKeyDownCapture" | "orientation" | "style"
> & {
  columns: number;
  className?: string;
  onKeyDownCapture?: (event: KeyboardEvent<HTMLDivElement>) => void;
  style?: CSSProperties;
};

const gridDirections: Partial<Record<string, GridNavigationDirection>> = {
  ArrowDown: "down",
  ArrowLeft: "left",
  ArrowRight: "right",
  ArrowUp: "up",
};

export function ButtonGrid({
  className,
  columns,
  onKeyDownCapture,
  style,
  ...props
}: ButtonGridProps) {
  const handleKeyDownCapture = (event: KeyboardEvent<HTMLDivElement>) => {
    onKeyDownCapture?.(event);
    if (event.defaultPrevented) return;

    const direction = gridDirections[event.key];
    if (!direction) return;

    const target = event.target;
    if (!(target instanceof Element)) return;

    const toolbar = event.currentTarget.querySelector<HTMLElement>(
      ":scope > [role='toolbar']",
    );
    if (!toolbar) return;

    const currentItem = target.closest<HTMLElement>("button, [role='button']");
    if (!currentItem || currentItem.parentElement !== toolbar) return;

    const items = Array.from(toolbar.children).filter(
      (child): child is HTMLElement =>
        child instanceof HTMLElement &&
        child.matches("button:not(:disabled), [role='button']") &&
        child.getAttribute("aria-disabled") !== "true",
    );
    const currentIndex = items.indexOf(currentItem);
    if (currentIndex < 0) return;

    const nextIndex = getNextGridItemIndex({
      columns,
      currentIndex,
      direction,
      itemCount: items.length,
    });

    event.preventDefault();
    event.stopPropagation();

    const nextItem = items[nextIndex];
    nextItem.focus({ preventScroll: true });
    nextItem.scrollIntoView({
      block: "nearest",
      inline: "nearest",
    });
  };

  return (
    <div className="contents" onKeyDownCapture={handleKeyDownCapture}>
      <AriaToolbar
        {...props}
        className={cn("grid [&>*]:scroll-m-focus-safe", className)}
        orientation="horizontal"
        style={{
          ...style,
          gridTemplateColumns: `repeat(${String(columns)}, minmax(0, 1fr))`,
        }}
      />
    </div>
  );
}
