// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";
import { TooltipTrigger } from "react-aria-components";

import { IconToggleButton } from "../../../components/base/button/icon-button";
import { Keyboard } from "../../../components/base/keyboard/keyboard";
import { Tooltip } from "../../../components/base/tooltip/tooltip";

/**
 * One Editor tool in the preview toolbar: a toggle whose tooltip
 * names its keyboard shortcut.
 */
export function PreviewToolToggle({
  children,
  isDisabled = false,
  isSelected,
  label,
  name,
  onSelectedChange,
  shortcut,
}: {
  children: ReactNode;
  isSelected: boolean;
  /** Tooltip wording, next to the shortcut key. */
  label: string;
  /** Accessible name, which says what the tool acts on. */
  name: string;
  onSelectedChange: (selected: boolean) => void;
  isDisabled?: boolean;
  shortcut?: string;
}) {
  return (
    <TooltipTrigger delay={400}>
      <span className="inline-flex">
        <IconToggleButton
          aria-keyshortcuts={shortcut}
          aria-label={name}
          isDisabled={isDisabled}
          isSelected={isSelected}
          onChange={onSelectedChange}
        >
          {children}
        </IconToggleButton>
      </span>
      <Tooltip placement="top">
        <span className="flex items-center gap-control-inset">
          {label}
          {shortcut ? <Keyboard>{shortcut}</Keyboard> : null}
        </span>
      </Tooltip>
    </TooltipTrigger>
  );
}
