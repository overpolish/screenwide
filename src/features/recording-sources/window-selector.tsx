// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { AppWindowMac } from "lucide-react";

import { Button } from "../../components/base/button/button";
import { ButtonGrid } from "../../components/base/button-group/button-group";
import { CircularProgress } from "../../components/base/circular-progress/circular-progress";
import { ScrollArea } from "../../components/base/scroll-area/scroll-area";
import { Text } from "../../components/base/text/text";
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
        <CircularProgress aria-label="Loading windows" isIndeterminate />
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
        <Text variant="subheadline">No Windows</Text>
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
    // app icon; the scroll fade runs to the window edge.
    <ScrollArea
      className="p-control-inset"
      rootClassName="rounded-window [--scrollbar-inset:var(--radius-window)]"
    >
      <ButtonGrid aria-label="Windows" columns={3}>
        {orderedWindows.map((window) => {
          const isSelected = selectedWindow?.id === window.id;

          return (
            // A thumbnail on no bezel; the chosen one wears an accent ring.
            <Button
              aria-label={`Select ${window.appName}: ${window.title}`}
              className="h-auto min-w-0 flex-col items-stretch justify-start gap-control px-control py-control"
              data-source-selector-focus-target={
                window.id === focusTargetId ? "true" : undefined
              }
              key={window.id}
              onPress={(event) => {
                onSelect(
                  window,
                  ["keyboard", "virtual"].includes(event.pointerType),
                );
              }}
              variant="ghost"
            >
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
              <span className="flex w-full min-w-0 items-center gap-control text-left">
                <AppIcon path={window.appIconPath} />
                <span className="min-w-0 truncate text-subheadline">
                  {window.title}
                </span>
              </span>
            </Button>
          );
        })}
      </ButtonGrid>
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
