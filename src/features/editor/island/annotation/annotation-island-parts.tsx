// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Trash2 } from "lucide-react";
import { type ReactNode } from "react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";
import { IconToggleButton } from "../../../../primitives/button/icon-button";
import { IslandFlow } from "../../../../primitives/island/island-flow";
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
 * leave the rest of the line empty beside it. A control that comes and
 * goes with a choice elsewhere comes into focus or blurs away in its place
 * while the others glide to theirs (`IslandFlow`). */
export function AnnotationLine({ children }: { children: ReactNode }) {
  return (
    <IslandFlow className="flex justify-between gap-separate has-[>:only-child]:justify-center">
      {children}
    </IslandFlow>
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

export function AnnotationDelete({ onDelete }: { onDelete: () => void }) {
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
