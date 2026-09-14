// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Copy, Minus, Square, X } from "lucide-react";
import { ReactNode } from "react";

import { detectedPlatform } from "../../../lib/platform";
import { cn } from "../../../lib/styling";
import { IconButton } from "../../base/button/icon-button";
import { ScrollArea } from "../../base/scroll-area/scroll-area";
import { Text } from "../../base/text/text";

import { EditableWindowTitle } from "./editable-window-title";

// macOS windows that show this header wear the real traffic lights, so the
// header leaves room for them and draws no window buttons of its own.

export type WindowHeaderProps = {
  title: string;
  actions?: ReactNode;
  /**
   * Tools carried in the title bar itself, centred in the window the way a
   * unified toolbar sets them. Given one, the bar becomes three columns so the
   * centre is the window's true middle and the title truncates to make room.
   */
  center?: ReactNode;
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
  center,
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
  const hasTrailing =
    Boolean(actions) ||
    Boolean(closeAction) ||
    (isWindows && Boolean(onMinimize || onToggleMaximize || onClose));

  const leading = (
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
  );

  // On macOS, controls overhang the title line into the inset. Windows uses
  // full-height caption controls so the bar follows the native 40px frame.
  const trailing = hasTrailing ? (
    <div
      className={cn(
        "flex shrink-0 items-center justify-end gap-section",
        isWindows && "self-stretch",
      )}
    >
      {actions ? (
        <div className="-my-tight flex shrink-0 items-center">{actions}</div>
      ) : null}
      {(isWindows && (onMinimize || onToggleMaximize || onClose)) ||
      closeAction ? (
        <div
          className={cn(
            "gap-control flex shrink-0 items-center",
            isWindows && "h-full gap-0",
          )}
        >
          {isWindows && onMinimize ? (
            <IconButton
              aria-label="Minimize"
              className="h-full w-[46px] rounded-none p-0"
              onPress={onMinimize}
            >
              <Minus />
            </IconButton>
          ) : null}
          {isWindows && onToggleMaximize ? (
            <IconButton
              aria-label={isMaximized ? "Restore" : "Maximize"}
              className="h-full w-[46px] rounded-none p-0"
              onPress={onToggleMaximize}
            >
              {isMaximized ? <Copy /> : <Square />}
            </IconButton>
          ) : null}
          {closeAction ??
            (isWindows && onClose ? (
              <IconButton
                aria-label="Close"
                className="h-full w-[46px] shrink-0 rounded-none p-0 data-[hovered]:bg-error data-[hovered]:text-primary-fg data-[pressed]:bg-error data-[pressed]:text-primary-fg"
                onPress={onClose}
              >
                <X />
              </IconButton>
            ) : null)}
        </div>
      ) : null}
    </div>
  ) : null;

  return (
    <header
      className={cn(
        // The title bar is the window inset around 24px controls on every
        // side, so the content below needs no top padding of its own.
        "shrink-0 items-center gap-section p-window-inset text-content-fg",
        isWindows && "h-10 p-0 pl-window-inset",
        // With tools in the bar the three columns are measured from the
        // window, not from the title: the centre stays put while the title
        // clips. Without them the bar is the flex row it has always been.
        center ? "grid grid-cols-[1fr_auto_1fr]" : "flex",
        !isWindows && "pl-traffic-lights",
        className,
      )}
      data-tauri-drag-region={isDraggable ? "deep" : undefined}
    >
      {leading}
      {center ? (
        <div className="-my-tight flex shrink-0 items-center justify-center gap-section">
          {center}
        </div>
      ) : null}
      {center ? (trailing ?? <div />) : trailing}
    </header>
  );
}
