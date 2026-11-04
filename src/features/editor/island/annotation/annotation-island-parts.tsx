// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Trash2 } from "lucide-react";
import {
  Children,
  isValidElement,
  type ReactElement,
  type ReactNode,
} from "react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { IconToggleButton } from "../../../../primitives/button/icon-button";
import { IslandParts } from "../../../../primitives/island/island-parts";
import { AnnotationColors } from "../../../annotations/options/annotation-colors";
import { DrawInIcon } from "../../glyphs/draw-in-icon";

// What every annotation's island holds whatever its kind: at the row's end,
// whether it draws itself in as its bar on the strip begins and deleting
// it, which asks twice as every delete on the island does; beneath, the
// colours it can be drawn in, a colour of the user's own among them, kept
// to pick again; and the lines its own controls sit on.

/** A line of an annotation's controls under its row. Several are spread
 * across it, the first at its start and the last at its end, as the rows
 * below keep theirs; one alone is centred, where at the start it would
 * leave the rest of the line empty beside it. Controls of different
 * heights, such as a position grid beside a pair of pills, centre on one
 * another. A control that comes and goes with a choice elsewhere comes
 * into focus or blurs away in its place while the others glide to theirs
 * (`IslandParts`). */
export function AnnotationLine({ children }: { children: ReactNode }) {
  return (
    <IslandParts
      className="flex items-center justify-between gap-separate has-[>:only-child]:justify-center"
      direction="row"
    >
      {children}
    </IslandParts>
  );
}

/** A choice between looks named at the start of its row, its glyphs at the
 * end, as the rows round it name themselves: where a lone line of glyphs
 * would float between options that each show their name. A part that comes
 * and goes beside the choice, such as an action for the look chosen, comes
 * into focus or blurs away as the island's parts do (`IslandParts`). */
export function AnnotationChoiceRow({
  children,
  title,
}: {
  children: ReactNode;
  title: string;
}) {
  return (
    <ControlRow title={title}>
      {(controlProps) => (
        <div {...controlProps} role="group">
          <IslandParts
            className="flex items-center gap-separate"
            direction="row"
          >
            {children}
          </IslandParts>
        </div>
      )}
    </ControlRow>
  );
}

/** Named rows that follow one another, set closer than the section's other
 * parts, as rows in a list are (`gap-inline`): their names are small type
 * in rows a control tall, so at the section's spacing they read as far
 * apart. A row that comes and goes does so as the island's parts do
 * (`IslandParts`). */
export function AnnotationRows({ children }: { children: ReactNode }) {
  return (
    <IslandParts className="flex flex-col gap-inline">{children}</IslandParts>
  );
}

/** Options that each name themselves, two to a line, so their words fit
 * the island where a line of them would run it wide. The lines share one
 * grid's columns (a subgrid each): the first as wide as its widest option
 * at the section's start, the second as wide as its widest at the end, and
 * between them the room left over, never less than the space between
 * neighbours. So the options sit flush with both edges and each column's
 * glyphs line up down the block. An option that comes and goes reflows the
 * lines after it, which swap as the island's parts do (`IslandParts`). */
export function AnnotationOptions({ children }: { children: ReactNode }) {
  // eslint-disable-next-line @eslint-react/no-children-to-array
  const options = Children.toArray(children).filter(isValidElement);
  const lines: ReactElement[][] = [];
  for (let index = 0; index < options.length; index += 2)
    lines.push(options.slice(index, index + 2));
  return (
    <IslandParts
      className="grid grid-cols-[auto_minmax(var(--spacing-separate),1fr)_auto] gap-y-inline"
      partClassName="col-span-3 grid grid-cols-subgrid"
    >
      {lines.map((line) => (
        <div
          className="col-span-3 grid grid-cols-subgrid justify-items-start [&>:nth-child(2)]:col-start-3"
          key={line.map((option) => String(option.key)).join("+")}
        >
          {line}
        </div>
      ))}
    </IslandParts>
  );
}

/** The colours of the user's own, and keeping or forgetting one. */
export type AnnotationColorProps = {
  onRemoveColor: (color: string) => void;
  onSaveColor: (color: string) => void;
  /** After the palette's, in the order they were kept. */
  savedColors: string[];
};

export function AnnotationColorChoice({
  color,
  onChange,
  onRemoveColor,
  onSaveColor,
  savedColors,
}: AnnotationColorProps & {
  color: string;
  onChange: (color: string) => void;
}) {
  return (
    <AnnotationColors
      custom={{ onRemove: onRemoveColor, onSave: onSaveColor }}
      onChange={onChange}
      savedColors={savedColors}
      value={color}
    />
  );
}

/** Deleting the annotation in hand, which asks twice; none for a pen taken
 * up with nothing chosen, whose island shows the style the next one is
 * drawn in. */
export function AnnotationDelete({ onDelete }: { onDelete?: () => void }) {
  if (!onDelete) return null;
  return (
    <ConfirmActionButton
      confirmLabel={t("editor-island-annotation-confirm-delete")}
      icon={<Trash2 aria-hidden />}
      label={t("editor-island-annotation-delete")}
      onConfirm={onDelete}
      variant="ghost"
    />
  );
}

/** Whether it draws itself in: a circle moved off with motion lines behind
 * it while it does, the circle alone at rest while it appears whole. */
export function AnnotationAnimateToggle({
  isAnimated,
  onChange,
}: {
  isAnimated: boolean;
  onChange: (isAnimated: boolean) => void;
}) {
  return (
    <IconToggleButton
      aria-label={t("editor-island-annotation-animate")}
      isSelected={isAnimated}
      onChange={onChange}
    >
      {({ isSelected }) => (
        <DrawInIcon className="size-icon" isMoving={isSelected} />
      )}
    </IconToggleButton>
  );
}
