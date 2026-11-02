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
// caption buttons, full height and square as the system draws them. A group
// in the middle (the editor's modes) stands centred while the title and the
// actions leave it room: the two grow alike from one width, the actions
// never less than they need. Short of room, the title gives way.

type WindowHeaderProps = {
  title: string;
  actions?: ReactNode;
  /** Centred between the title and the actions, such as the editor's
   * modes. */
  center?: ReactNode;
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
  center,
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
        className={cn(
          "pointer-events-none min-w-0 truncate",
          center != null ? "grow basis-24" : "flex-1",
        )}
        data-tauri-drag-region={isDraggable ? "" : undefined}
        dir="auto"
        variant="heading"
      >
        {onTitleChange ? (
          <EditableTitle
            // Only as wide as the name, so the rest of the bar still moves
            // the window; a long name truncates rather than taking it all.
            className="w-fit max-w-[min(100%,28rem)] min-w-4 truncate focus:text-clip"
            label={t("controls-document-name")}
            onChange={onTitleChange}
            title={title}
          />
        ) : (
          title
        )}
      </Text>
      {center}
      {actions || center != null ? (
        <div
          className={cn(
            "flex items-center gap-inline",
            center != null ? "min-w-max grow basis-24 justify-end" : "shrink-0",
          )}
        >
          {actions}
        </div>
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
