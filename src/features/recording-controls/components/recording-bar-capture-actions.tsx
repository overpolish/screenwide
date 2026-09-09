// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  getCurrentWindow,
  LogicalPosition,
  LogicalSize,
} from "@tauri-apps/api/window";
import {
  Check,
  Circle,
  ImageDown,
  Lock,
  OctagonAlert,
  PanelsTopLeft,
} from "lucide-react";
import { useEffect, useMemo, useRef } from "react";

import { Button } from "../../../components/base/button/button";
import { IconButton } from "../../../components/base/button/icon-button";
import { cn } from "../../../lib/styling";
import { hidePopupPanel, showPopupPanel } from "../../popup-panel/api";
import { initialPopupPanelHeight } from "../../popup-panel/layout";
import { PopupPanelItem, usePopupPanelStore } from "../../popup-panel/store";
import { ScreenshotState } from "../types";

/** Identifies this trigger in the shared popup panel window. */
const SCREENSHOT_MENU_ID = "recording-bar-screenshot";
/** The alternate actions read wider than the button they hang off. */
const SCREENSHOT_MENU_MIN_WIDTH = 200;

type RecordingBarCaptureActionsProps = {
  canRecord: boolean;
  canScreenshot: boolean;
  canScrollingScreenshot: boolean;
  isCapturing: boolean;
  isLocked: boolean;
  isRecordBlockedByEditor: boolean;
  className?: string;
  onFocusPendingEditor?: () => void;
  onRecord?: () => void;
  onRequiredPermissionsPress?: () => void;
  onScreenshot?: () => void;
  onScreenshotToClipboard?: () => void;
  onScrollingScreenshot?: () => void;
  screenshotState?: ScreenshotState;
};

export function RecordingBarCaptureActions({
  canRecord,
  canScreenshot,
  canScrollingScreenshot,
  className,
  isCapturing,
  isLocked,
  isRecordBlockedByEditor,
  onFocusPendingEditor,
  onRecord,
  onRequiredPermissionsPress,
  onScreenshot,
  onScreenshotToClipboard,
  onScrollingScreenshot,
  screenshotState = "idle",
}: RecordingBarCaptureActionsProps) {
  const close = usePopupPanelStore((state) => state.close);
  const lastSelection = usePopupPanelStore((state) => state.lastSelection);
  const open = usePopupPanelStore((state) => state.open);
  const handledEventRef = useRef(lastSelection?.eventId ?? null);

  const items = useMemo<PopupPanelItem[]>(
    () => [
      { icon: "image", id: "save", label: "Save Screenshot" },
      { icon: "clipboard", id: "clipboard", label: "Copy to Clipboard" },
      ...(canScrollingScreenshot
        ? [
            {
              icon: "scrolling" as const,
              id: "scrolling",
              label: "Capture Scrolling Region",
            },
          ]
        : []),
    ],
    [canScrollingScreenshot],
  );

  useEffect(() => {
    if (
      !lastSelection ||
      lastSelection.id !== SCREENSHOT_MENU_ID ||
      lastSelection.eventId === handledEventRef.current
    ) {
      return;
    }

    handledEventRef.current = lastSelection.eventId;
    const { active, close: closeListbox } = usePopupPanelStore.getState();
    // The panel is a menu, not a select: picking an action dismisses it.
    if (active?.id === SCREENSHOT_MENU_ID) {
      closeListbox();
      void hidePopupPanel(active.focusContents);
    }

    const selected = lastSelection.selectedIds[0];
    if (selected === "save") onScreenshot?.();
    else if (selected === "clipboard") onScreenshotToClipboard?.();
    else if (selected === "scrolling") onScrollingScreenshot?.();
  }, [
    lastSelection,
    onScreenshot,
    onScreenshotToClipboard,
    onScrollingScreenshot,
  ]);

  const showMenu = async (anchor: DOMRect, focusContents: boolean) => {
    const current = usePopupPanelStore.getState().active;
    if (current?.id === SCREENSHOT_MENU_ID) {
      close();
      await hidePopupPanel(current.focusContents);
      return;
    }

    open({
      focusContents,
      id: SCREENSHOT_MENU_ID,
      items,
      label: "Screenshot options",
      mode: "menu",
      selectedIds: [],
      selectionMode: "single",
    });
    await showPopupPanel({
      anchor: {
        height: anchor.height,
        width: anchor.width,
        x: anchor.left,
        y: anchor.top,
      },
      focusContents,
      offset: new LogicalPosition(anchor.left, anchor.bottom + 4),
      parentWindowLabel: getCurrentWindow().label,
      size: new LogicalSize(
        Math.max(anchor.width, SCREENSHOT_MENU_MIN_WIDTH),
        initialPopupPanelHeight(items.length),
      ),
      triggerId: SCREENSHOT_MENU_ID,
    });
  };

  const isScreenshotDisabled = !canScreenshot || isCapturing;

  return (
    <div className={cn("flex items-center gap-control-inset", className)}>
      {/* Marked as a trigger so closing the panel with the keyboard returns
          focus to the button rather than to the window. */}
      <div data-popup-panel-trigger={SCREENSHOT_MENU_ID}>
        <IconButton
          aria-label="Screenshot"
          isDisabled={isScreenshotDisabled}
          onPress={(event) => {
            void showMenu(
              event.target.getBoundingClientRect(),
              ["keyboard", "virtual"].includes(event.pointerType),
            );
          }}
          size="capture"
        >
          {/* Status is carried by the glyph alone, as the alert does. */}
          {screenshotState === "done" ? (
            <Check className="text-success" />
          ) : screenshotState === "failed" ? (
            <OctagonAlert className="text-error" />
          ) : (
            <ImageDown
              className={cn(
                isCapturing && "animate-pulse text-content-fg-secondary",
              )}
            />
          )}
        </IconButton>
      </div>

      <Button
        aria-label={
          isLocked
            ? "Open permissions"
            : isRecordBlockedByEditor
              ? "Show Editor"
              : "Record"
        }
        color="primary"
        isDisabled={!canRecord && !isRecordBlockedByEditor && !isLocked}
        onPress={
          isLocked
            ? onRequiredPermissionsPress
            : isRecordBlockedByEditor
              ? onFocusPendingEditor
              : onRecord
        }
        size="capture"
      >
        {isLocked ? (
          <Lock />
        ) : isRecordBlockedByEditor ? (
          <PanelsTopLeft />
        ) : (
          <Circle />
        )}
      </Button>
    </div>
  );
}

export type { RecordingBarCaptureActionsProps };
