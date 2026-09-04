// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { X } from "lucide-react";
import { MotionProps } from "motion/react";
import { use } from "react";
import { SelectStateContext } from "react-aria-components";

import { IconButton } from "../../button/icon-button";

type ClearButtonProps = MotionProps & {
  isDisabled?: boolean;
  onClear?: () => void;
  size?: "compact" | "default";
};

export const ClearButton = ({
  isDisabled,
  onClear,
  size,
  ...props
}: ClearButtonProps) => {
  const state = use(SelectStateContext);

  if (!state?.selectedItems.length) return null;

  return (
    <div className="flex shrink-0 items-center" data-select-clear>
      <IconButton
        {...props}
        aria-label="Clear selection"
        isDisabled={isDisabled}
        onPress={() => {
          state.setValue(null);
          if (onClear) onClear();
        }}
        size={size}
        slot={null}
      >
        <X />
      </IconButton>
    </div>
  );
};
