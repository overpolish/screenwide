// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Folder, RotateCcw } from "lucide-react";
import { Button } from "react-aria-components";

import { t } from "../../i18n/i18n";
import { folderName } from "../../lib/path-label";
import { cn, elementFocusVisible } from "../../lib/styling";
import { IconButton } from "../../primitives/button/icon-button";
import { fieldVariants } from "../../primitives/field/field-variants";
import { TooltipTrigger } from "../../primitives/tooltip/tooltip-trigger";

// Shows the folder's name, with its full path in the tooltip. The host opens
// the system's picker and keeps the choice.

type PathFieldProps = {
  "aria-label": string;
  onBrowse: () => void;
  /** Full path; `null` before one is chosen. */
  value: string | null;
  /** Width; `w-40` when left out. */
  className?: string;
  isDisabled?: boolean;
  /** "Choose folder" when left out. */
  placeholder?: string;
} & (
  | { onReset: () => void; resetLabel: string }
  | { onReset?: never; resetLabel?: never }
);

const fieldStyles = cn(
  fieldVariants({ material: "raised" }).field(),
  "w-full min-w-0 cursor-default gap-inline px-inset text-body",
  "data-[hovered]:control-raised-hover data-[pressed]:control-raised-pressed data-[disabled]:opacity-40",
  elementFocusVisible,
);

export function PathField({
  "aria-label": label,
  className,
  isDisabled,
  onBrowse,
  onReset,
  placeholder,
  resetLabel,
  value,
}: PathFieldProps) {
  const empty = placeholder ?? t("controls-path-field-choose");
  return (
    <div className="flex min-w-0 items-center gap-inline">
      {/* Before the field, so the field never moves when it appears. */}
      {onReset && value !== null ? (
        <IconButton
          aria-label={resetLabel}
          isDisabled={isDisabled}
          onPress={onReset}
          variant="ghost"
        >
          <RotateCcw />
        </IconButton>
      ) : null}
      <TooltipTrigger
        className={cn("w-40 min-w-0", className)}
        isDisabled={value === null}
        tooltip={value ?? ""}
      >
        <Button
          aria-label={t("controls-path-field", {
            label,
            path: value ?? empty,
          })}
          className={fieldStyles}
          isDisabled={isDisabled}
          onPress={onBrowse}
        >
          <Folder
            aria-hidden="true"
            className="size-icon shrink-0 text-fg-secondary"
          />
          <span
            className={cn(
              "min-w-0 truncate",
              value === null ? "text-fg-tertiary" : "text-fg",
            )}
            dir="auto"
          >
            {value === null ? empty : folderName(value)}
          </span>
        </Button>
      </TooltipTrigger>
    </div>
  );
}
