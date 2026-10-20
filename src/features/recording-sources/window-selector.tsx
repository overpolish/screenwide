// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { AppWindowMac } from "lucide-react";
import { ListBoxItem } from "react-aria-components";

import { buttonVariants } from "../../components/base/button/button-variants";
import { CircularProgress } from "../../components/base/circular-progress/circular-progress";
import { ListBox } from "../../components/base/listbox/listbox";
import { ScrollArea } from "../../components/base/scroll-area/scroll-area";
import { Text } from "../../components/base/text/text";
import { t } from "../../i18n/i18n";
import { cn } from "../../lib/styling";

import { WindowDetails } from "./types";

type WindowSelectorProps = {
  error: string | null;
  isLoading: boolean;
  onSelect: (window: WindowDetails, returnFocus: boolean) => void;
  selectedWindow: WindowDetails | null;
  windows: WindowDetails[];
};

export function WindowSelector({
  error,
  isLoading,
  onSelect,
  selectedWindow,
  windows,
}: WindowSelectorProps) {
  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <CircularProgress
          aria-label={t("recording-sources-loading-windows")}
          isIndeterminate
        />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-full items-center justify-center p-window-inset text-center">
        <Text className="text-error" variant="subheadline">
          {error}
        </Text>
      </div>
    );
  }

  if (windows.length === 0) {
    return (
      <div className="flex h-full items-center justify-center">
        <Text variant="subheadline">{t("recording-sources-no-windows")}</Text>
      </div>
    );
  }

  const orderedWindows = [...windows].sort(compareWindows);
  const focusTargetId = orderedWindows.some(
    (window) => window.id === selectedWindow?.id,
  )
    ? selectedWindow?.id
    : orderedWindows[0]?.id;

  return (
    // One grid, ordered by app then title, with each caption carrying the
    // app icon; the scroll fade runs to the window edge. A list box rather
    // than buttons, so the chosen window is announced as selected and the
    // arrow keys move by where the thumbnails sit.
    <ScrollArea
      className="p-control-inset"
      rootClassName="rounded-window [--scrollbar-inset:var(--radius-window)]"
    >
      <ListBox
        aria-label={t("recording-sources-windows")}
        className="grid w-full grid-cols-3 gap-control-inset overflow-visible p-0 windows:gap-control-inset [&>*]:scroll-m-focus-safe"
        layout="grid"
        selectedKeys={selectedWindow ? [selectedWindow.id] : []}
        selectionBehavior="replace"
        selectionMode="single"
      >
        {orderedWindows.map((window) => (
          <ListBoxItem
            aria-label={t("recording-sources-select-window", {
              app: window.appName,
              title: window.title,
            })}
            // A thumbnail on no bezel, drawn as the ghost button it replaces.
            className={buttonVariants({
              className:
                "h-auto min-w-0 flex-col items-stretch justify-start gap-control px-control py-control",
              variant: "ghost",
            })}
            data-source-selector-focus-target={
              window.id === focusTargetId ? "true" : undefined
            }
            id={window.id}
            key={window.id}
            // Press rather than selection change, so choosing the window that
            // is already chosen still closes the selector, as a click does.
            onPress={(event) => {
              onSelect(
                window,
                ["keyboard", "virtual"].includes(event.pointerType),
              );
            }}
            textValue={window.title}
          >
            {({ isSelected }) => (
              <>
                <span
                  className={cn(
                    "flex aspect-video min-h-0 w-full items-center justify-center overflow-hidden rounded-control bg-fill-quaternary",
                    isSelected && "ring-3 ring-primary",
                  )}
                >
                  {window.thumbnailPath ? (
                    <img
                      alt=""
                      className="max-h-full max-w-full object-contain"
                      src={convertFileSrc(window.thumbnailPath)}
                    />
                  ) : (
                    <span className="flex text-content-fg-tertiary">
                      <AppWindowMac className="size-icon-large" />
                    </span>
                  )}
                </span>
                <span className="flex w-full min-w-0 items-center gap-control text-start">
                  <AppIcon path={window.appIconPath} />
                  <span
                    className="min-w-0 truncate text-subheadline"
                    dir="auto"
                  >
                    {window.title}
                  </span>
                </span>
              </>
            )}
          </ListBoxItem>
        ))}
      </ListBox>
    </ScrollArea>
  );
}

function AppIcon({ path }: { path: string | null }) {
  return path ? (
    <img
      alt=""
      className="size-icon shrink-0 object-contain"
      src={convertFileSrc(path)}
    />
  ) : (
    <AppWindowMac className="size-icon shrink-0" />
  );
}

const compareWindows = (left: WindowDetails, right: WindowDetails) => {
  const appOrder = left.appName.localeCompare(right.appName, undefined, {
    sensitivity: "base",
  });
  if (appOrder !== 0) return appOrder;
  return left.title.localeCompare(right.title, undefined, {
    sensitivity: "base",
  });
};
