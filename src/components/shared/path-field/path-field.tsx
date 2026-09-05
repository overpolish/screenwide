// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { File, Folder, RotateCcw, X } from "lucide-react";
import { TooltipTrigger } from "react-aria-components";

import { truncatePath } from "../../../lib/path-label";
import { cn } from "../../../lib/styling";
import { Button } from "../../base/button/button";
import { IconButton } from "../../base/button/icon-button";
import { Tooltip } from "../../base/tooltip/tooltip";

export type PathFieldProps = {
  "aria-label": string;
  onBrowse: () => void;
  value: string | null;
  className?: string;
  emptyLabel?: string;
  isDisabled?: boolean;
  kind?: "file" | "folder";
  maxLabelLength?: number;
  secondaryAction?: {
    label: string;
    onPress: () => void;
    type: "clear" | "reset";
  };
};

/** The host owns native pickers and persistence. */
export function PathField({
  "aria-label": label,
  className,
  emptyLabel,
  isDisabled,
  kind = "folder",
  maxLabelLength = 30,
  onBrowse,
  secondaryAction,
  value,
}: PathFieldProps) {
  const display = value
    ? truncatePath(value, maxLabelLength, kind)
    : (emptyLabel ?? `Choose ${kind}`);
  return (
    <div
      className={cn("gap-control-inset inline-flex items-center", className)}
    >
      <TooltipTrigger isDisabled={!value}>
        <Button
          aria-label={`${label}: ${value || display}`}
          isDisabled={isDisabled}
          onPress={onBrowse}
        >
          {kind === "folder" ? <Folder /> : <File />}
          <span className="whitespace-nowrap">{display}</span>
        </Button>
        <Tooltip className="max-w-80 break-words">{value}</Tooltip>
      </TooltipTrigger>
      {secondaryAction ? (
        <IconButton
          aria-label={secondaryAction.label}
          isDisabled={isDisabled || !value}
          onPress={secondaryAction.onPress}
        >
          {secondaryAction.type === "reset" ? <RotateCcw /> : <X />}
        </IconButton>
      ) : null}
    </div>
  );
}
