// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { MouseEvent as ReactMouseEvent, ReactNode } from "react";
import { TooltipTrigger } from "react-aria-components";

import { ToggleButton } from "../../../components/base/button/toggle-button";
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
        <ToggleButton
          animation="scale-selected"
          aria-keyshortcuts={shortcut}
          aria-label={name}
          isSelected={isSelected}
          onChange={onSelectedChange}
          showFocus={false}
          size="sm"
          variant="ghost"
        >
          {children}
        </ToggleButton>
      </span>
      <Tooltip placement="bottom">
        <span className="flex items-center gap-2">
          {label}
          {shortcut ? (
            <Keyboard size="xs" variant="tooltip">
              {shortcut}
            </Keyboard>
          ) : null}
        </span>
      </Tooltip>
    </TooltipTrigger>
  );
}
