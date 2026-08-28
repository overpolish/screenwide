// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Lock, TriangleAlert } from "lucide-react";
import { ReactNode } from "react";

import { IconToggleButton } from "../../../components/base/button/icon-button";
import { cn } from "../../../lib/styling";

type RecordingBarInputToggleProps = {
  isSelected: boolean;
  label: string;
  off: ReactNode;
  on: ReactNode;
  onChange: (isSelected: boolean) => void;
  hasWarning?: boolean;
  isDisabled?: boolean;
  isLocked?: boolean;
  isReadOnly?: boolean;
  onLockedPress?: () => void;
  warningLabel?: string;
};

export function RecordingBarInputToggle({
  hasWarning,
  isDisabled,
  isLocked,
  isReadOnly,
  isSelected,
  label,
  off,
  on,
  onChange,
  onLockedPress,
  warningLabel,
}: RecordingBarInputToggleProps) {
  return (
    <div className="relative flex justify-center">
      {hasWarning && isSelected && !isDisabled ? (
        <TriangleAlert
          aria-label={warningLabel ?? `${label} source is not detected`}
          className="size-icon-indicator pointer-events-none absolute top-0 z-10 -translate-y-1/2 text-warning"
          role="img"
        />
      ) : isLocked && !isDisabled ? (
        <Lock className="size-icon-indicator pointer-events-none absolute top-0 z-10 -translate-y-1/2 text-muted" />
      ) : null}
      <IconToggleButton
        aria-disabled={isReadOnly || undefined}
        aria-label={label}
        className={cn(
          isReadOnly &&
            "pointer-events-none cursor-default data-[hovered]:scale-100",
        )}
        isDisabled={isDisabled}
        isSelected={isSelected}
        off={off}
        onChange={(selected) => {
          if (isReadOnly) return;
          if (isLocked) {
            onLockedPress?.();
          } else {
            onChange(selected);
          }
        }}
        size="compact"
      >
        {on}
      </IconToggleButton>
    </div>
  );
}
