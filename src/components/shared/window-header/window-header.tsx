// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Copy, Minus, Square, X } from "lucide-react";
import { ReactNode } from "react";

import { cn } from "../../../lib/styling";
import { IconButton } from "../../base/button/icon-button";

export type WindowHeaderProps = {
  title: string;
  actions?: ReactNode;
  className?: string;
  isMaximized?: boolean;
  leadingSection?: ReactNode;
  onClose?: () => void;
  onMinimize?: () => void;
  onToggleMaximize?: () => void;
  variant?: "compact" | "display";
};

export function WindowHeader({
  actions,
  className,
  isMaximized = false,
  leadingSection,
  onClose,
  onMinimize,
  onToggleMaximize,
  title,
  variant = "display",
}: WindowHeaderProps) {
  return (
    <header
      className={cn(
        "gap-section px-window-inset pt-window-inset flex shrink-0 items-start text-content-fg",
        variant === "display" && "items-center",
        className,
      )}
      data-tauri-drag-region="deep"
    >
      <div
        className={cn(
          "gap-control-inset pointer-events-none flex min-w-0 grow items-start",
          variant === "display" && "gap-section items-center",
        )}
        data-tauri-drag-region
      >
        {leadingSection ? (
          <span
            className={cn(
              "flex h-6 shrink-0 items-center [&_img]:size-icon-default [&_svg]:size-icon-default",
              variant === "display" &&
                "h-9 [&_img]:size-icon-prominent [&_svg]:size-icon-prominent",
            )}
          >
            {leadingSection}
          </span>
        ) : null}
        <h1
          className={cn(
            "min-w-0 text-base font-semibold",
            variant === "display" &&
              "from-accent-heading-warm via-accent-heading to-accent-heading-vivid animate-gradient bg-linear-to-r bg-clip-text bg-size-[300%] text-3xl font-bold text-transparent motion-reduce:animate-none",
          )}
        >
          {title}
        </h1>
      </div>
      {actions}
      {onMinimize || onToggleMaximize || onClose ? (
        <div className="gap-control flex shrink-0 items-center">
          {onMinimize ? (
            <IconButton
              aria-label="Minimize"
              onPress={onMinimize}
              size="compact"
            >
              <Minus />
            </IconButton>
          ) : null}
          {onToggleMaximize ? (
            <IconButton
              aria-label={isMaximized ? "Restore" : "Maximize"}
              onPress={onToggleMaximize}
              size="compact"
            >
              {isMaximized ? <Copy /> : <Square />}
            </IconButton>
          ) : null}
          {onClose ? (
            <IconButton
              aria-label="Close"
              className="shrink-0"
              onPress={onClose}
              size="compact"
            >
              <X />
            </IconButton>
          ) : null}
        </div>
      ) : null}
    </header>
  );
}
