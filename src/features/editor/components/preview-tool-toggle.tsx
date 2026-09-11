// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { IconToggleButton } from "../../../components/base/button/icon-button";
import { NativeTooltipTrigger } from "../../../components/shared/native-tooltip/native-tooltip-trigger";

/**
 * One Editor tool in the preview toolbar: a toggle whose tooltip
 * names its keyboard shortcut.
 *
 * The tooltip is a window of its own: the preview it stands over is a native
 * surface layered above the webview, which nothing drawn in the page can
 * appear over.
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
    <NativeTooltipTrigger tooltip={{ label, shortcut }}>
      <IconToggleButton
        aria-keyshortcuts={shortcut}
        aria-label={name}
        isDisabled={isDisabled}
        isSelected={isSelected}
        onChange={onSelectedChange}
      >
        {children}
      </IconToggleButton>
    </NativeTooltipTrigger>
  );
}
