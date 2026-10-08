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
  History,
  ImageDown,
  Lock,
  OctagonAlert,
} from "lucide-react";
import { useEffect, useMemo, useRef } from "react";

import { Button } from "../../../components/base/button/button";
import {
  IconButton,
  IconToggleButton,
} from "../../../components/base/button/icon-button";
import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { hidePopupPanel, showPopupPanel } from "../../popup-panel/api";
import { initialPopupPanelHeight } from "../../popup-panel/layout";
import {
  activePopupPanel,
  PopupPanelItem,
  SHARED_POPUP_PANEL,
  usePopupPanelStore,
} from "../../popup-panel/store";
import { ReplaySnapshot, ScreenshotState } from "../types";

/** Identifies this trigger in the shared popup panel window. */
const SCREENSHOT_MENU_ID = "recording-bar-screenshot";
/** The alternate actions read wider than the button they hang off. */
const SCREENSHOT_MENU_MIN_WIDTH = 200;

type RecordingBarCaptureActionsProps = {
  canRecord: boolean;
  canScreenshot: boolean;
  canScrollingScreenshot: boolean;
  /** Whether the bar's settings could start the replay buffer now. */
  canStartReplay: boolean;
  isCapturing: boolean;
  isLocked: boolean;
  className?: string;
  onDelayedScreenshot?: () => void;
  onRecord?: () => void;
  onReplayChange?: (on: boolean) => void;
  onRequiredPermissionsPress?: () => void;
  onScreenshot?: () => void;
  onScreenshotToClipboard?: () => void;
  onScrollingScreenshot?: () => void;
  replay?: ReplaySnapshot;
  screenshotState?: ScreenshotState;
};

export function RecordingBarCaptureActions({
  canRecord,
  canScreenshot,
  canScrollingScreenshot,
  canStartReplay,
  className,
  isCapturing,
  isLocked,
  onDelayedScreenshot,
  onRecord,
  onReplayChange,
  onRequiredPermissionsPress,
  onScreenshot,
  onScreenshotToClipboard,
  onScrollingScreenshot,
  replay,
  screenshotState = "idle",
}: RecordingBarCaptureActionsProps) {
  const close = usePopupPanelStore((state) => state.close);
  const lastSelection = usePopupPanelStore((state) => state.lastSelection);
  const open = usePopupPanelStore((state) => state.open);
  const handledEventRef = useRef(lastSelection?.eventId ?? null);

  const items = useMemo<PopupPanelItem[]>(
    () => [
      {
        icon: "image",
        id: "save",
        label: t("recording-controls-save-screenshot"),
      },
      {
        icon: "clipboard",
        id: "clipboard",
        label: t("recording-controls-copy-screenshot"),
      },
      {
        icon: "timer",
        id: "delayed",
        label: t("recording-controls-delayed-screenshot"),
      },
      ...(canScrollingScreenshot
        ? [
            {
              icon: "scrolling" as const,
              id: "scrolling",
              label: t("recording-controls-scrolling-screenshot"),
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
    const state = usePopupPanelStore.getState();
    const active = activePopupPanel(state, SHARED_POPUP_PANEL);
    // The panel is a menu, not a select: picking an action dismisses it.
    if (active?.id === SCREENSHOT_MENU_ID) {
      state.close(SHARED_POPUP_PANEL);
      void hidePopupPanel(active.focusContents, SHARED_POPUP_PANEL);
    }

    const selected = lastSelection.selectedIds[0];
    if (selected === "save") onScreenshot?.();
    else if (selected === "clipboard") onScreenshotToClipboard?.();
    else if (selected === "delayed") onDelayedScreenshot?.();
    else if (selected === "scrolling") onScrollingScreenshot?.();
  }, [
    lastSelection,
    onDelayedScreenshot,
    onScreenshot,
    onScreenshotToClipboard,
    onScrollingScreenshot,
  ]);

  const showMenu = async (anchor: DOMRect, focusContents: boolean) => {
    const current = activePopupPanel(
      usePopupPanelStore.getState(),
      SHARED_POPUP_PANEL,
    );
    if (current?.id === SCREENSHOT_MENU_ID) {
      close(SHARED_POPUP_PANEL);
      await hidePopupPanel(current.focusContents, SHARED_POPUP_PANEL);
      return;
    }

    open(SHARED_POPUP_PANEL, {
      content: {
        items,
        kind: "list",
        mode: "menu",
        selectedIds: [],
        selectionMode: "single",
      },
      focusContents,
      id: SCREENSHOT_MENU_ID,
      label: t("recording-controls-screenshot-options"),
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
          aria-label={t("recording-controls-screenshot")}
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
            ? t("recording-controls-open-permissions")
            : t("recording-controls-record")
        }
        color="primary"
        isDisabled={!canRecord && !isLocked}
        onPress={isLocked ? onRequiredPermissionsPress : onRecord}
        size="capture"
      >
        {isLocked ? <Lock /> : <Circle />}
      </Button>

      {replay?.available ? (
        <IconToggleButton
          aria-label={t("recording-controls-replay", {
            seconds: replay.lengthSeconds,
          })}
          // Starting needs settings the bar could record with; turning it off
          // is always allowed, whatever the bar is set to now.
          isDisabled={
            replay.status === "starting" ||
            (replay.status === "off" && !canStartReplay)
          }
          isSelected={replay.status !== "off"}
          onChange={onReplayChange}
          size="capture"
        >
          <History
            className={cn(
              replay.status === "starting" &&
                "animate-pulse text-content-fg-secondary",
            )}
          />
        </IconToggleButton>
      ) : null}
    </div>
  );
}

export type { RecordingBarCaptureActionsProps };
