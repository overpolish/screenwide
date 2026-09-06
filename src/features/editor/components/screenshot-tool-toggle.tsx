// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { MouseEvent as ReactMouseEvent, ReactNode } from "react";
import { TooltipTrigger } from "react-aria-components";

import { IconToggleButton } from "../../../components/base/button/icon-button";
import { Keyboard } from "../../../components/base/keyboard/keyboard";
import { Tooltip } from "../../../components/base/tooltip/tooltip";

/**
 * One screenshot editing tool in the preview toolbar: a toggle whose tooltip
 * names its keyboard shortcut and whose right-click resets what it edits.
 */
export function ScreenshotToolToggle({
  children,
  isSelected,
  label,
  name,
  onReset,
  onSelectedChange,
  shortcut,
}: {
  children: ReactNode;
  isSelected: boolean;
  /** Tooltip wording, next to the shortcut key. */
  label: string;
  /** Accessible name, which says what the tool acts on. */
  name: string;
  onReset: () => void;
  onSelectedChange: (selected: boolean) => void;
  shortcut?: string;
}) {
  return (
    <TooltipTrigger delay={400}>
      <span
        className="inline-flex"
        onContextMenu={(event: ReactMouseEvent<HTMLSpanElement>) => {
          event.preventDefault();
          onReset();
        }}
      >
        <IconToggleButton
          aria-keyshortcuts={shortcut}
          aria-label={name}
          isSelected={isSelected}
          onChange={onSelectedChange}
          size="compact"
        >
          {children}
        </IconToggleButton>
      </span>
      <Tooltip placement="bottom">
        <span className="flex items-center gap-2">
          {label}
          {shortcut ? <Keyboard>{shortcut}</Keyboard> : null}
        </span>
      </Tooltip>
    </TooltipTrigger>
  );
}
