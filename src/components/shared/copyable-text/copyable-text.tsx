// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Copy } from "lucide-react";
import { ComponentProps, useId } from "react";

import { cn } from "../../../lib/styling";
import { IconButton } from "../../base/button/icon-button";
import { ScrollArea } from "../../base/scroll-area/scroll-area";
import { Text } from "../../base/text/text";
import { CheckOnClick } from "../check-on-click/check-on-click";

type CopyableTextProps = Omit<ComponentProps<"section">, "children"> & {
  label: string;
  onCopy: () => unknown;
  value: string;
  emptyText?: string;
};

export function CopyableText({
  className,
  emptyText = "No Content",
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
        "flex min-h-0 flex-col gap-control overflow-hidden rounded-control bg-layer inset-ring inset-ring-layer-stroke p-section",
        className,
      )}
      {...props}
    >
      <div className="flex items-center justify-between gap-control-inset">
        <Text as="span" id={labelId} variant="section">
          {label}
        </Text>
        <CheckOnClick onPress={onCopy}>
          <IconButton
            aria-label={`Copy ${label.toLocaleLowerCase()}`}
            isDisabled={!value}
          >
            <Copy aria-hidden />
          </IconButton>
        </CheckOnClick>
      </div>

      {/* Long content scrolls inside the box rather than growing it. The
          scroll view bleeds to the box edges and the padding scrolls with the
          content, so text clips at the edge as a native scroll view does. */}
      <ScrollArea
        className="px-section pb-section"
        constrainHeight
        rootClassName="-mx-section -mb-section w-auto max-h-40 min-h-20"
      >
        {value ? (
          <pre className="font-sans text-body text-content-fg break-words whitespace-pre-wrap select-text">
            {value}
          </pre>
        ) : (
          <Text variant="subheadline">{emptyText}</Text>
        )}
      </ScrollArea>
    </section>
  );
}

export type { CopyableTextProps };
