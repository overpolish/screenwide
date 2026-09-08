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
  controlClassName?: string;
  description?: string;
};

/** Spread controlProps onto the actual control, not its layout wrapper.
 * The parent owns spacing between rows; the row itself is not clickable.
 */
export function Setting({
  children,
  className,
  controlClassName,
  description,
  title,
}: SettingProps) {
  const id = useId();
  const titleId = `${id}-title`;
  const descriptionId = description ? `${id}-description` : undefined;

  return (
    // A settings row as System Settings lays one out: title with its
    // description directly beneath, and the control trailing.
    <div className={cn("flex items-center gap-layout", className)}>
      <div className="flex min-w-0 flex-1 flex-col gap-tight">
        <Text className="break-words" id={titleId}>
          {title}
        </Text>
        {description ? (
          <Text
            className="break-words"
            id={descriptionId}
            variant="subheadline"
          >
            {description}
          </Text>
        ) : null}
      </div>
      <div className={cn("flex shrink-0 items-center", controlClassName)}>
        {children({
          "aria-describedby": descriptionId,
          "aria-labelledby": titleId,
        })}
      </div>
    </div>
  );
}
