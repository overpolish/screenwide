// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Copy, Minus, Square, X } from "lucide-react";
import { ReactNode } from "react";

import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { IconButton } from "../../base/button/icon-button";
import { ScrollArea } from "../../base/scroll-area/scroll-area";
import { Text } from "../../base/text/text";
import { EditableTitle } from "../editable-title/editable-title";

import {
  useWindowHeaderCenterWidth,
  WindowHeaderCenterWidthContext,
} from "./window-header-center-width";

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
  /**
   * Tools carried in the title bar itself, centred in the window the way a
   * unified toolbar sets them. Given one, the bar becomes three columns so the
   * centre is the window's true middle and the title truncates to make room,
   * down to `window-title`. The room the tools may take is published through
   * `WindowHeaderCenterWidthContext`, for tools that fold away to fit it.
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
  const centerWidth = useWindowHeaderCenterWidth(Boolean(center));

  const leading = (
    <div
      className={cn(
        "pointer-events-none flex grow items-center gap-control-inset",
        // With tools in the bar the title keeps a readable minimum; the tools
        // fold away before it is squeezed further.
        center ? "min-w-window-title" : "min-w-0",
      )}
      data-tauri-drag-region={isDraggable ? "" : undefined}
      ref={centerWidth.leadingRef}
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
        // A document's name runs in whichever direction it is written.
        dir="auto"
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
          className={cn(variant === "display" && "text-accent-heading")}
          variant="title"
        >
          {onTitleChange ? (
            <EditableTitle
              label={t("controls-document-name")}
              onChange={onTitleChange}
              title={title}
            />
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
        // Content-sized in the grid, so the centre's measurement reads what
        // the side needs rather than the column it was given.
        center && "justify-self-end",
      )}
      ref={centerWidth.trailingRef}
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
              aria-label={t("controls-window-minimize")}
              className="h-full w-[46px] rounded-none p-0"
              onPress={onMinimize}
            >
              <Minus />
            </IconButton>
          ) : null}
          {isWindows && onToggleMaximize ? (
            <IconButton
              aria-label={
                isMaximized
                  ? t("controls-window-restore")
                  : t("controls-window-maximize")
              }
              className="h-full w-[46px] rounded-none p-0"
              onPress={onToggleMaximize}
            >
              {isMaximized ? <Copy /> : <Square />}
            </IconButton>
          ) : null}
          {closeAction ??
            (isWindows && onClose ? (
              <IconButton
                aria-label={t("controls-window-close")}
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
        isWindows && "h-10 p-0 ps-window-inset",
        // With tools in the bar the three columns are measured from the
        // window, not from the title: the centre stays put while the title
        // clips. Without them the bar is the flex row it has always been.
        center ? "grid grid-cols-[1fr_auto_1fr]" : "flex",
        !isWindows && "ps-traffic-lights",
        className,
      )}
      data-tauri-drag-region={isDraggable ? "deep" : undefined}
      ref={centerWidth.headerRef}
    >
      {leading}
      {center ? (
        <div className="-my-tight flex shrink-0 items-center justify-center gap-section">
          <WindowHeaderCenterWidthContext value={centerWidth.width}>
            {center}
          </WindowHeaderCenterWidthContext>
        </div>
      ) : null}
      {center ? (trailing ?? <div />) : trailing}
    </header>
  );
}
