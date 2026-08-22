// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ArrowBigDownDash,
  Check,
  ClipboardCopy,
  ImageDown,
} from "lucide-react";

import { Button } from "../../../components/base/button/button";
import { cn } from "../../../lib/styling";
import { ScreenshotState } from "../types";

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
  size,
  state,
}: {
  icon: typeof ImageDown;
  isCapturing: boolean;
  size: number;
  state: ScreenshotState;
}) {
  return state === "done" ? (
    <Check className="text-success" size={size} strokeWidth={3} />
  ) : (
    <Icon
      className={cn(
        "origin-center transform-gpu backface-hidden will-change-transform transition-[color,transform,scale] group-data-[hovered]:scale-110",
        isCapturing && "animate-pulse text-muted",
        state === "failed" && "text-error",
      )}
      size={size}
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
  return (
    <div className="mr-3 flex flex-col items-center justify-center self-stretch">
      <Button
        aria-label="Take screenshot"
        className="group cursor-default p-1"
        isDisabled={!canExportScreenshot || isCapturingStill}
        onPress={onScreenshot}
        showFocus={false}
        variant="ghost"
      >
        <FeedbackIcon
          icon={ImageDown}
          isCapturing={isCapturingStill}
          size={40}
          state={exportScreenshotState}
        />
      </Button>

      <div className="flex items-center justify-center">
        <Button
          aria-label="Copy screenshot to clipboard"
          className="group cursor-default"
          isDisabled={!canCopyScreenshot || isCapturingStill}
          onPress={onScreenshotToClipboard}
          showFocus={false}
          size="sm"
          variant="ghost"
        >
          <FeedbackIcon
            icon={ClipboardCopy}
            isCapturing={isCapturingStill}
            size={16}
            state={clipboardScreenshotState}
          />
        </Button>

        <Button
          aria-label="Capture scrolling region"
          className="group cursor-default"
          isDisabled={!canCaptureScrollingScreenshot || isCapturingStill}
          onPress={onScrollingScreenshot}
          showFocus={false}
          size="sm"
          variant="ghost"
        >
          <FeedbackIcon
            icon={ArrowBigDownDash}
            isCapturing={isCapturingStill}
            size={16}
            state={scrollingScreenshotState}
          />
        </Button>
      </div>
    </div>
  );
}
