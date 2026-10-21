// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Copy, Minus, Square, X } from "lucide-react";
import { ReactNode } from "react";

import { t } from "../../i18n/i18n";
import { hostPlatform, type Platform } from "../../lib/platform";
import { cn } from "../../lib/styling";
import { IconButton } from "../../primitives/button/icon-button";
import { Text } from "../../primitives/text/text";
import { EditableTitle } from "../editable-title/editable-title";

// The same bar on both platforms: macOS draws its traffic lights over the
// start, which the bar leaves free; Windows has none, so the bar ends in the
// caption buttons, full height and square as the system draws them.

type WindowHeaderProps = {
  title: string;
  actions?: ReactNode;
  /** Off for a window the system places, such as a sheet. */
  isDraggable?: boolean;
  isMaximized?: boolean;
  onClose?: () => void;
  onMinimize?: () => void;
  onTitleChange?: (title: string) => void;
  onToggleMaximize?: () => void;
  /** Stories only; a window follows the platform it runs on. */
  platform?: Platform;
};

const captionButton = "h-full w-12 rounded-none";

export function WindowHeader({
  actions,
  isDraggable = true,
  isMaximized = false,
  onClose,
  onMinimize,
  onTitleChange,
  onToggleMaximize,
  platform = hostPlatform,
  title,
}: WindowHeaderProps) {
  const isWindows = platform === "windows";
  const hasCaption =
    isWindows && Boolean(onMinimize || onToggleMaximize || onClose);

  return (
    <header
      className={cn(
        "flex h-12 shrink-0 items-center gap-separate",
        isWindows ? "ps-inset" : "ps-traffic-lights",
        !hasCaption && "pe-inset",
      )}
      data-tauri-drag-region={isDraggable ? "deep" : undefined}
    >
      <Text
        as="h1"
        className="pointer-events-none min-w-0 flex-1 truncate"
        data-tauri-drag-region={isDraggable ? "" : undefined}
        dir="auto"
        variant="heading"
      >
        {onTitleChange ? (
          <EditableTitle
            className="truncate focus:text-clip"
            label={t("controls-document-name")}
            onChange={onTitleChange}
            title={title}
          />
        ) : (
          title
        )}
      </Text>
      {actions ? (
        <div className="flex shrink-0 items-center gap-inline">{actions}</div>
      ) : null}
      {hasCaption ? (
        <div className="flex h-full shrink-0">
          {onMinimize ? (
            <IconButton
              aria-label={t("controls-window-minimize")}
              className={captionButton}
              onPress={onMinimize}
              variant="ghost"
            >
              <Minus />
            </IconButton>
          ) : null}
          {onToggleMaximize ? (
            <IconButton
              aria-label={
                isMaximized
                  ? t("controls-window-restore")
                  : t("controls-window-maximize")
              }
              className={captionButton}
              onPress={onToggleMaximize}
              variant="ghost"
            >
              {isMaximized ? <Copy /> : <Square />}
            </IconButton>
          ) : null}
          {onClose ? (
            <IconButton
              aria-label={t("controls-window-close")}
              // Close turns red under the pointer, as the system's does.
              className={cn(
                captionButton,
                "data-[hovered]:bg-error data-[hovered]:text-white data-[pressed]:bg-error data-[pressed]:text-white",
              )}
              onPress={onClose}
              variant="ghost"
            >
              <X />
            </IconButton>
          ) : null}
        </div>
      ) : null}
    </header>
  );
}
