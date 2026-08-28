// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { AppWindowMac, CircleSlash2 } from "lucide-react";

import { Button } from "../../components/base/button/button";
import { ButtonGrid } from "../../components/base/button-group/button-group";
import { CircularProgress } from "../../components/base/circular-progress/circular-progress";
import { OverflowShadow } from "../../components/base/overflow-shadow/overflow-shadow";

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
      <div className="flex h-full items-center justify-center text-center text-xs text-error">
        {error}
      </div>
    );
  }

  if (windows.length === 0) {
    return (
      <div className="gap-section flex h-full items-center justify-center text-xs font-semibold text-muted">
        <CircleSlash2 className="size-icon-prominent" />
        No windows found
      </div>
    );
  }

  const orderedWindows = [...windows].sort((left, right) => {
    const appOrder = left.appName.localeCompare(right.appName, undefined, {
      sensitivity: "base",
    });
    if (appOrder !== 0) return appOrder;

    return left.title.localeCompare(right.title, undefined, {
      sensitivity: "base",
    });
  });
  const focusTargetId = orderedWindows.some(
    (window) => window.id === selectedWindow?.id,
  )
    ? selectedWindow?.id
    : orderedWindows[0]?.id;

  return (
    <OverflowShadow orientation="vertical" rootClassName="rounded-xl">
      <ButtonGrid aria-label="Windows" className="gap-control" columns={3}>
        {orderedWindows.map((window) => {
          const isSelected = selectedWindow?.id === window.id;

          return (
            <Button
              aria-label={`Select ${window.appName}: ${window.title}`}
              className="gap-control min-w-0 flex-col items-stretch justify-start"
              color={isSelected ? "primary" : "neutral"}
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
            >
              <span className="flex aspect-video min-h-0 w-full items-center justify-center overflow-hidden rounded-lg">
                {window.thumbnailPath ? (
                  <img
                    alt=""
                    className="max-h-full max-w-full rounded-lg object-contain"
                    src={convertFileSrc(window.thumbnailPath)}
                  />
                ) : (
                  <span className="gap-control flex flex-col items-center text-xs">
                    <AppWindowMac className="size-icon-prominent" />
                    Preview unavailable
                  </span>
                )}
              </span>

              <span className="gap-control flex w-full min-w-0 items-center text-left">
                {window.appIconPath ? (
                  <img
                    alt=""
                    className="size-icon-compact shrink-0 object-contain"
                    src={convertFileSrc(window.appIconPath)}
                  />
                ) : (
                  <AppWindowMac className="size-icon-compact shrink-0" />
                )}
                <span className="min-w-0 truncate text-xs font-medium">
                  {window.title}
                </span>
              </span>
            </Button>
          );
        })}
      </ButtonGrid>
    </OverflowShadow>
  );
}
