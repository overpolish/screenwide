// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { X } from "lucide-react";
import { ReactNode } from "react";

import { IconButton } from "../../base/button/icon-button";

export type WindowHeaderProps = {
  title: string;
  actions?: ReactNode;
  description?: string;
  onClose?: () => void;
};

export function WindowHeader({
  actions,
  description,
  onClose,
  title,
}: WindowHeaderProps) {
  return (
    <header
      className="gap-section px-window-inset pt-window-inset pb-section flex shrink-0 items-start"
      data-tauri-drag-region="deep"
    >
      <div className="pointer-events-none min-w-0 grow" data-tauri-drag-region>
        <h1 className="text-base font-semibold">{title}</h1>
        {description ? (
          <p className="mt-tight text-xs text-muted">{description}</p>
        ) : null}
      </div>
      {actions}
      {onClose ? (
        <IconButton
          aria-label="Close"
          className="shrink-0"
          onPress={onClose}
          size="compact"
        >
          <X />
        </IconButton>
      ) : null}
    </header>
  );
}
