// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useId, type ReactNode } from "react";

import { cn } from "../../../lib/styling";
import { Text } from "../../base/text/text";

export type ControlRowControlProps = {
  "aria-labelledby": string;
  "aria-describedby"?: string;
};

export type ControlRowProps = {
  children: (controlProps: ControlRowControlProps) => ReactNode;
  title: string;
  className?: string;
  controlClassName?: string;
  description?: string;
  /** An icon tile or glyph before the text, as the Privacy list shows one. */
  leading?: ReactNode;
  /** Something beside the title, such as a badge. */
  titleAccessory?: ReactNode;
};

/** Spread controlProps onto the actual control, not its layout wrapper.
 * The parent owns spacing between rows; the row itself is not clickable.
 */
export function ControlRow({
  children,
  className,
  controlClassName,
  description,
  leading,
  title,
  titleAccessory,
}: ControlRowProps) {
  const id = useId();
  const titleId = `${id}-title`;
  const descriptionId = description ? `${id}-description` : undefined;

  return (
    // A labelled control row as System Settings and inspector panels lay one
    // out: title with its description directly beneath, and the control
    // trailing.
    <div className={cn("flex items-center gap-layout", className)}>
      {leading ? (
        <div className="-mr-section flex shrink-0 items-center">{leading}</div>
      ) : null}
      <div className="flex min-w-0 flex-1 flex-col gap-tight">
        <div className="flex items-center gap-control">
          <Text className="break-words" id={titleId}>
            {title}
          </Text>
          {titleAccessory}
        </div>
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
