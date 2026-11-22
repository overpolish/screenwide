// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, ImageDown, OctagonAlert } from "lucide-react";

import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { IconButton } from "../../../primitives/button/icon-button";
import { IconSwap } from "../../../primitives/icon-swap/icon-swap";
import { IslandOptions } from "../../../primitives/island/options/island-options";

import type { ScreenshotState } from "../types";

// The island's screenshot: pressed, a still of what is recorded; its options
// open the other kinds. The same at rest and while recording, where a still
// is taken beside the recording.

export function ScreenshotButton({
  className,
  isDisabled = false,
  isOpen,
  onOpen,
  onScreenshot,
  state,
}: {
  isOpen: boolean;
  onOpen: () => void;
  className?: string;
  isDisabled?: boolean;
  onScreenshot?: () => void;
  state?: ScreenshotState;
}) {
  return (
    <IslandOptions
      isDisabled={isDisabled}
      isOpen={isOpen}
      label={t("recording-controls-screenshot-options")}
      onOpen={onOpen}
    >
      <IconButton
        aria-label={t("recording-controls-screenshot")}
        className={cn(state === "pending" && "text-fg-secondary", className)}
        isDisabled={isDisabled}
        onPress={onScreenshot}
        variant="ghost"
      >
        {/* Status is carried by the glyph alone, and the glyphs crossfade as
            it changes. */}
        <IconSwap
          swapKey={state === "done" || state === "failed" ? state : "ready"}
        >
          {state === "done" ? (
            <span className="flex text-success">
              <Check aria-hidden className="size-icon" />
            </span>
          ) : state === "failed" ? (
            <span className="flex text-error">
              <OctagonAlert aria-hidden className="size-icon" />
            </span>
          ) : (
            <ImageDown
              aria-hidden
              className={cn(
                "size-icon",
                state === "pending" && "animate-pulse",
              )}
            />
          )}
        </IconSwap>
      </IconButton>
    </IslandOptions>
  );
}
