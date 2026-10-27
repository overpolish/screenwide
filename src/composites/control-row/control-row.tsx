// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useId, type ReactNode } from "react";

import { cn } from "../../lib/styling";
import { useSkin } from "../../primitives/skin/skin";
import { Text } from "../../primitives/text/text";

export type ControlRowControlProps = {
  "aria-labelledby": string;
  "aria-describedby"?: string;
};

export type ControlRowProps = {
  /** Spread the props onto the control itself, not a wrapper around it. */
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

// A setting's row, as System Settings and inspector panels lay one out: its
// name, a note under it when the name needs one, and the control at the end,
// which the name and note label for assistive technology. At least a
// control's height, so a row reads the same whether or not it has a note.
// The note is set back, in the caption type and a quieter colour, so it reads
// as a note rather than a second name. The parent owns the space between
// rows; the row itself is not pressable. On the island, where a section holds
// a few short rows under the row that names them, the name is a caption set
// back, and the row is an island control's height.
export function ControlRow({
  children,
  className,
  controlClassName,
  description,
  leading,
  title,
  titleAccessory,
}: ControlRowProps) {
  const isIsland = useSkin() === "island";
  const id = useId();
  const titleId = `${id}-title`;
  const descriptionId = description ? `${id}-description` : undefined;

  return (
    <div
      className={cn(
        "flex items-center gap-separate",
        isIsland ? "min-h-island-control" : "min-h-control-height",
        className,
      )}
      // Lets a search result find the row it names.
      data-control-row={title}
    >
      {leading ? (
        <div className="flex shrink-0 items-center">{leading}</div>
      ) : null}
      <div className="flex min-w-0 flex-1 flex-col">
        <div className="flex items-center gap-inline">
          <Text
            className={cn("break-words", isIsland && "ps-1 text-fg-secondary")}
            id={titleId}
            variant={isIsland ? "caption" : "body"}
          >
            {title}
          </Text>
          {titleAccessory}
        </div>
        {description ? (
          <Text className="break-words" id={descriptionId} variant="caption">
            {description}
          </Text>
        ) : null}
      </div>
      <div
        className={cn(
          "flex shrink-0 items-center gap-inline",
          controlClassName,
        )}
      >
        {children({
          "aria-describedby": descriptionId,
          "aria-labelledby": titleId,
        })}
      </div>
    </div>
  );
}
