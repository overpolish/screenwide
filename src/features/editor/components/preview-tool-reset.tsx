// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RotateCcw } from "lucide-react";

import { IconButton } from "../../../components/base/button/icon-button";
import { NativeTooltipTrigger } from "../../../components/shared/native-tooltip/native-tooltip-trigger";

const resetLabels = {
  canvas: "Reset canvas",
  crop: "Reset crop",
  recenter: "Reset recenter",
  select: "Reset transform",
};

export function PreviewToolReset({
  canvasLabel = "Reset canvas",
  isDisabled = false,
  onReset,
  tool,
}: {
  tool: keyof typeof resetLabels | null;
  canvasLabel?: string;
  isDisabled?: boolean;
  onReset?: () => void;
}) {
  const label =
    tool === "canvas" ? canvasLabel : tool ? resetLabels[tool] : "Reset tool";
  return (
    <span
      aria-hidden={tool === null || undefined}
      className={`absolute top-1/2 left-1/2 inline-flex -translate-x-1/2 -translate-y-1/2 ${tool === null ? "invisible" : ""}`}
    >
      <NativeTooltipTrigger tooltip={label}>
        <IconButton
          aria-label={label}
          isDisabled={tool === null || isDisabled || !onReset}
          onPress={onReset}
        >
          <RotateCcw />
        </IconButton>
      </NativeTooltipTrigger>
    </span>
  );
}
