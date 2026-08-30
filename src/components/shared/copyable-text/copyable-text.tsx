// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Copy } from "lucide-react";
import { ComponentProps, useId } from "react";

import { cn } from "../../../lib/styling";
import { IconButton } from "../../base/button/icon-button";
import { CheckOnClick } from "../check-on-click/check-on-click";

type CopyableTextProps = Omit<ComponentProps<"section">, "children"> & {
  label: string;
  onCopy: () => unknown;
  value: string;
  emptyText?: string;
};

export function CopyableText({
  className,
  emptyText = "(empty)",
  label,
  onCopy,
  value,
  ...props
}: CopyableTextProps) {
  const labelId = useId();

  return (
    <section
      aria-labelledby={labelId}
      className={cn(
        "gap-control p-section flex min-h-0 flex-col overflow-hidden rounded-xl bg-neutral",
        className,
      )}
      {...props}
    >
      <div className="gap-control-inset flex items-center justify-between">
        <div className="text-xs font-bold text-muted" id={labelId}>
          {label}
        </div>
        <CheckOnClick onPress={onCopy}>
          <IconButton
            aria-label={`Copy ${label.toLocaleLowerCase()}`}
            isDisabled={!value}
            size="compact"
          >
            <Copy aria-hidden />
          </IconButton>
        </CheckOnClick>
      </div>

      <pre className="min-h-20 grow overflow-auto font-mono text-sm leading-relaxed break-words whitespace-pre-wrap select-text">
        {value || emptyText}
      </pre>
    </section>
  );
}

export type { CopyableTextProps };
