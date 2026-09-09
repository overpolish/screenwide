// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Copy, Minus, Square, X } from "lucide-react";
import { ReactNode } from "react";

import { cn } from "../../../lib/styling";
import { IconButton } from "../../base/button/icon-button";
import { ScrollArea } from "../../base/scroll-area/scroll-area";
import { Text } from "../../base/text/text";

import { EditableWindowTitle } from "./editable-window-title";

/**
 * The platform this build runs on, read the same way `main.tsx` reads it.
 * macOS windows that show this header wear the real traffic lights, so the
 * header leaves room for them and draws no window buttons of its own.
 */
const detectedPlatform: "macos" | "windows" = navigator.userAgent.includes(
  "Windows",
)
  ? "windows"
  : "macos";

export type WindowHeaderProps = {
  title: string;
  actions?: ReactNode;
  className?: string;
  closeAction?: ReactNode;
  /** Off for windows the OS positions itself, such as a child sheet. */
  isDraggable?: boolean;
  isMaximized?: boolean;
  leadingSection?: ReactNode;
  onClose?: () => void;
  onMinimize?: () => void;
  onTitleChange?: (title: string) => void;
  onToggleMaximize?: () => void;
  /** Stories only: the real windows follow the platform they run on. */
  platform?: "macos" | "windows";
  variant?: "compact" | "display";
};

export function WindowHeader({
  actions,
  className,
  closeAction,
  isDraggable = true,
  isMaximized = false,
  leadingSection,
  onClose,
  onMinimize,
  onTitleChange,
  onToggleMaximize,
  platform = detectedPlatform,
  title,
  variant = "display",
}: WindowHeaderProps) {
  const isWindows = platform === "windows";

  return (
    <header
      className={cn(
        // The title bar is the window inset around 24px controls on every
        // side, so the content below needs no top padding of its own.
        "flex shrink-0 items-center gap-section p-window-inset text-content-fg",
        !isWindows && "pl-traffic-lights",
        className,
      )}
      data-tauri-drag-region={isDraggable ? "deep" : undefined}
    >
      <div
        className="pointer-events-none flex min-w-0 grow items-center gap-control-inset"
        data-tauri-drag-region={isDraggable ? "" : undefined}
      >
        {/* The proxy icon of a native title bar is drawn at the standard
            symbol size. */}
        {leadingSection ? (
          <span className="flex shrink-0 items-center [&_img]:size-icon [&_svg]:size-icon">
            {leadingSection}
          </span>
        ) : null}
        <ScrollArea
          className="flex w-max items-center"
          edgeClassName="rounded-control"
          edgeEffect="shadow"
          orientation="horizontal"
          // A plain title is part of the drag region like the logo; only an
          // editable one takes the pointer.
          rootClassName={cn(
            "w-max min-w-0 max-w-full shrink",
            onTitleChange && "pointer-events-auto",
          )}
          scrollbarHidden
        >
          <Text
            as="h1"
            className={cn(
              variant === "display" &&
                "from-accent-heading-warm via-accent-heading to-accent-heading-vivid animate-gradient bg-linear-to-r bg-clip-text bg-size-[300%] text-transparent motion-reduce:animate-none",
            )}
            variant="title"
          >
            {onTitleChange ? (
              <EditableWindowTitle onChange={onTitleChange} title={title} />
            ) : (
              title
            )}
          </Text>
        </ScrollArea>
      </div>
      {actions}
      {(isWindows && (onMinimize || onToggleMaximize || onClose)) ||
      closeAction ? (
        <div className="gap-control flex shrink-0 items-center">
          {isWindows && onMinimize ? (
            <IconButton aria-label="Minimize" onPress={onMinimize}>
              <Minus />
            </IconButton>
          ) : null}
          {isWindows && onToggleMaximize ? (
            <IconButton
              aria-label={isMaximized ? "Restore" : "Maximize"}
              onPress={onToggleMaximize}
            >
              {isMaximized ? <Copy /> : <Square />}
            </IconButton>
          ) : null}
          {closeAction ??
            (isWindows && onClose ? (
              <IconButton
                aria-label="Close"
                className="shrink-0"
                onPress={onClose}
              >
                <X />
              </IconButton>
            ) : null)}
        </div>
      ) : null}
    </header>
  );
}
