// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode, use } from "react";

import { IconToggleButton } from "../button/icon-button";
import { TooltipTrigger } from "../tooltip/tooltip-trigger";

import { ToolbarDisabledContext } from "./toolbar-disabled";

// The tooltip is a window of its own: the surfaces these toolbars stand over,
// the Editor's preview and the live annotation overlay, are native layers
// above the webview, which nothing drawn in the page can appear over.
export function ToolbarTool({
  "aria-label": ariaLabel,
  children,
  isDisabled = false,
  isSelected,
  label,
  onChange,
  shortcut,
}: {
  /** Says what the tool acts on, where `label` only names it. */
  "aria-label": string;
  children: ReactNode;
  isSelected: boolean;
  /** Tooltip wording, beside the shortcut. */
  label: string;
  onChange: (isSelected: boolean) => void;
  isDisabled?: boolean;
  shortcut?: string;
}) {
  const isBarDisabled = use(ToolbarDisabledContext);
  return (
    <TooltipTrigger tooltip={{ label, shortcut }}>
      <IconToggleButton
        aria-keyshortcuts={shortcut}
        aria-label={ariaLabel}
        isDisabled={isDisabled || isBarDisabled}
        isSelected={isSelected}
        onChange={onChange}
      >
        {children}
      </IconToggleButton>
    </TooltipTrigger>
  );
}
