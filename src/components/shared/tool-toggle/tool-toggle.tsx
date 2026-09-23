// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode, use } from "react";

import { IconToggleButton } from "../../base/button/icon-button";
import { NativeTooltipTrigger } from "../native-tooltip/native-tooltip-trigger";

import { ToolToggleDisabledContext } from "./tool-toggle-disabled";

/**
 * One tool in a toolbar: a toggle whose tooltip names its keyboard shortcut.
 *
 * The tooltip is a window of its own, because the surfaces these toolbars
 * stand over - the Editor's preview, the live annotation overlay - are native
 * layers above the webview, which nothing drawn in the page can appear over.
 */
export function ToolToggle({
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
  const isBarDisabled = use(ToolToggleDisabledContext);
  return (
    <NativeTooltipTrigger tooltip={{ label, shortcut }}>
      <IconToggleButton
        aria-keyshortcuts={shortcut}
        aria-label={name}
        isDisabled={isDisabled || isBarDisabled}
        isSelected={isSelected}
        onChange={onSelectedChange}
      >
        {children}
      </IconToggleButton>
    </NativeTooltipTrigger>
  );
}
