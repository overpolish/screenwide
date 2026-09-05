// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useId, type ReactNode } from "react";

import { cn } from "../../../lib/styling";
import { Text } from "../../base/text/text";

export type SettingControlProps = {
  "aria-labelledby": string;
  "aria-describedby"?: string;
};

export type SettingProps = {
  children: (controlProps: SettingControlProps) => ReactNode;
  title: string;
  className?: string;
  description?: string;
};

/** Spread controlProps onto the actual control, not its layout wrapper.
 * The parent owns spacing between rows; the row itself is not clickable.
 */
export function Setting({
  children,
  className,
  description,
  title,
}: SettingProps) {
  const id = useId();
  const titleId = `${id}-title`;
  const descriptionId = description ? `${id}-description` : undefined;

  return (
    <div className={cn("gap-layout flex items-center", className)}>
      <div className="gap-control flex min-w-0 flex-1 flex-col">
        <Text className="break-words" id={titleId}>
          {title}
        </Text>
        {description ? (
          <Text className="break-words" id={descriptionId} variant="help">
            {description}
          </Text>
        ) : null}
      </div>
      <div className="flex shrink-0 items-center">
        {children({
          "aria-describedby": descriptionId,
          "aria-labelledby": titleId,
        })}
      </div>
    </div>
  );
}
