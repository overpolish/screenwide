// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Dices } from "lucide-react";

import { IconButton } from "../../base/button/icon-button";
import { Switch } from "../../base/switch/switch";
import { NativeTooltipTrigger } from "../native-tooltip/native-tooltip-trigger";

/**
 * Whether a highlight is drawn by hand. While it is, the dice draws the stroke
 * again from a fresh seed; it is only offered where there is a stroke in hand
 * to redraw, so the live overlay's toolbar, which dresses the next stroke,
 * leaves `onRandomise` out.
 */
export function AnnotationHandDrawnToggle({
  isDisabled,
  isSelected,
  onChange,
  onRandomise,
}: {
  isSelected: boolean;
  onChange: (handDrawn: boolean) => void;
  isDisabled?: boolean;
  onRandomise?: () => void;
}) {
  return (
    <div className="flex items-center gap-control">
      {isSelected && onRandomise ? (
        <NativeTooltipTrigger tooltip={{ label: "Randomise" }}>
          <IconButton
            aria-label="Randomise"
            isDisabled={isDisabled}
            onPress={onRandomise}
          >
            <Dices />
          </IconButton>
        </NativeTooltipTrigger>
      ) : null}
      <Switch
        aria-label="Hand-drawn"
        isDisabled={isDisabled}
        isSelected={isSelected}
        onChange={onChange}
      />
    </div>
  );
}
