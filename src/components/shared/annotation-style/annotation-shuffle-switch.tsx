// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Dices } from "lucide-react";

import { IconButton } from "../../base/button/icon-button";
import { Switch } from "../../base/switch/switch";
import { NativeTooltipTrigger } from "../native-tooltip/native-tooltip-trigger";

/**
 * A switch for something drawn from a seed: a hand-drawn stroke, an image's
 * sway. While it is on, the dice draws it again from a fresh seed; the dice
 * is only offered where there is something in hand to redraw, so a control
 * that dresses the next annotation leaves `onRandomise` out.
 */
export function AnnotationShuffleSwitch({
  isDisabled,
  isSelected,
  label,
  onChange,
  onRandomise,
}: {
  isSelected: boolean;
  label: string;
  onChange: (isSelected: boolean) => void;
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
        aria-label={label}
        isDisabled={isDisabled}
        isSelected={isSelected}
        onChange={onChange}
      />
    </div>
  );
}
