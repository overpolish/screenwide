// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentProps, useId } from "react";

import { CopyButton } from "../../composites/copy-button/copy-button";
import { t } from "../../i18n/i18n";
import { cn } from "../../lib/styling";
import { ScrollArea } from "../../primitives/scroll-area/scroll-area";

type CopyableTextProps = Omit<ComponentProps<"section">, "children"> & {
  /** Names the copy button, such as "Copy detected content". */
  copyLabel: string;
  label: string;
  onCopy: () => unknown;
  value: string;
  emptyText?: string;
};

// Text found in a picture, to read and to copy. It is read-only text, so it
// sits in a well of the field material, as a field's value would, under its
// heading with the copy button at the end of the heading's line. The text
// can be selected. Long text scrolls inside the well, fading at the edge
// where there is more.
export function CopyableText({
  className,
  copyLabel,
  emptyText = t("controls-no-content"),
  label,
  onCopy,
  value,
  ...props
}: CopyableTextProps) {
  const labelId = useId();

  return (
    <section
      aria-labelledby={labelId}
      className={cn("flex min-h-0 flex-col gap-inline", className)}
      {...props}
    >
      {/* On the row's foot, so the heading sits as close over the well as
          the button does, rather than centred higher up. */}
      <div className="flex items-end justify-between gap-inline">
        <span className="text-label text-fg-secondary" id={labelId}>
          {label}
        </span>
        <CopyButton
          aria-label={copyLabel}
          isDisabled={!value}
          onCopy={onCopy}
        />
      </div>
      <ScrollArea
        className="p-inset-sm"
        constrainHeight
        rootClassName="max-h-40 min-h-10 rounded-control control-sunken"
      >
        {value ? (
          <pre className="font-sans text-body break-words whitespace-pre-wrap text-fg select-text">
            {value}
          </pre>
        ) : (
          <p className="text-body text-fg-tertiary">{emptyText}</p>
        )}
      </ScrollArea>
    </section>
  );
}

export type { CopyableTextProps };
