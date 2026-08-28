// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ArrowBigDownDash,
  Check,
  ClipboardCopy,
  ImageDown,
} from "lucide-react";

import { IconButton } from "../../../components/base/button/icon-button";
import { cn } from "../../../lib/styling";
import { ScreenshotState } from "../types";

const screenshotFailurePreviewEnabled =
  import.meta.env.DEV &&
  import.meta.env.VITE_SCREENWIDE_SCREENSHOT_FAILURE_PREVIEW === "1";

const failedActionClassName =
  "bg-error-surface text-error data-[hovered]:bg-error-surface-hover data-[pressed]:bg-error-surface-pressed";

type RecordingBarScreenshotActionsProps = {
  canCaptureScrollingScreenshot: boolean;
  canCopyScreenshot: boolean;
  canExportScreenshot: boolean;
  clipboardScreenshotState: ScreenshotState;
  exportScreenshotState: ScreenshotState;
  isCapturingStill: boolean;
  scrollingScreenshotState: ScreenshotState;
  onScreenshot?: () => void;
  onScreenshotToClipboard?: () => void;
  onScrollingScreenshot?: () => void;
};

function FeedbackIcon({
  icon: Icon,
  isCapturing,
  state,
}: {
  icon: typeof ImageDown;
  isCapturing: boolean;
  state: ScreenshotState;
}) {
  return state === "done" ? (
    <Check className="text-success" strokeWidth={3} />
  ) : (
    <Icon
      className={cn(
        "transition-colors",
        isCapturing && "animate-pulse text-muted",
      )}
    />
  );
}

export function RecordingBarScreenshotActions({
  canCaptureScrollingScreenshot,
  canCopyScreenshot,
  canExportScreenshot,
  clipboardScreenshotState,
  exportScreenshotState,
  isCapturingStill,
  onScreenshot,
  onScreenshotToClipboard,
  onScrollingScreenshot,
  scrollingScreenshotState,
}: RecordingBarScreenshotActionsProps) {
  const effectiveClipboardState = screenshotFailurePreviewEnabled
    ? "failed"
    : clipboardScreenshotState;
  const effectiveExportState = screenshotFailurePreviewEnabled
    ? "failed"
    : exportScreenshotState;
  const effectiveScrollingState = screenshotFailurePreviewEnabled
    ? "failed"
    : scrollingScreenshotState;

  return (
    <div className="gap-tight flex flex-col items-center justify-center self-stretch">
      <IconButton
        aria-label="Take screenshot"
        className={
          effectiveExportState === "failed" ? failedActionClassName : undefined
        }
        iconSize="prominent"
        isDisabled={!canExportScreenshot || isCapturingStill}
        onPress={onScreenshot}
      >
        <FeedbackIcon
          icon={ImageDown}
          isCapturing={isCapturingStill}
          state={effectiveExportState}
        />
      </IconButton>

      <div className="gap-tight flex items-center justify-center">
        <IconButton
          aria-label="Copy screenshot to clipboard"
          className={
            effectiveClipboardState === "failed"
              ? failedActionClassName
              : undefined
          }
          isDisabled={!canCopyScreenshot || isCapturingStill}
          onPress={onScreenshotToClipboard}
          size="compact"
        >
          <FeedbackIcon
            icon={ClipboardCopy}
            isCapturing={isCapturingStill}
            state={effectiveClipboardState}
          />
        </IconButton>

        <IconButton
          aria-label="Capture scrolling region"
          className={
            effectiveScrollingState === "failed"
              ? failedActionClassName
              : undefined
          }
          isDisabled={!canCaptureScrollingScreenshot || isCapturingStill}
          onPress={onScrollingScreenshot}
          size="compact"
        >
          <FeedbackIcon
            icon={ArrowBigDownDash}
            isCapturing={isCapturingStill}
            state={effectiveScrollingState}
          />
        </IconButton>
      </div>
    </div>
  );
}
