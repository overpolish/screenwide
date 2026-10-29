// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Link, Unlink } from "lucide-react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";

import { t } from "../../../../i18n/i18n";
import { IconToggleButton } from "../../../../primitives/button/icon-button";
import { NumberField } from "../../../../primitives/input-fields/number-field";
import { useSkin } from "../../../../primitives/skin/skin";

import {
  AspectRatioParts,
  dimensionsAtRatio,
  matchesRatio,
  reduceToRatio,
} from "./aspect-ratio-math";

export type DimensionsChange = { height?: number; width?: number };

type DimensionsFieldProps = {
  /** In pixels. */
  height: number;
  /** A changed side, or both when the aspect ratio is kept. */
  onChange: (change: DimensionsChange) => void;
  /** In pixels. */
  width: number;
  isDisabled?: boolean;
};

// A size as two fields with a toggle between them that keeps the aspect
// ratio: while it is on, editing one side moves the other. It is a row's
// control, named by the row it sits in. It starts with the ratio kept, as a
// picture is usually resized whole.

/** The figures a side keeps room for on the island, "99,999" as grouped:
 * any size a picture takes. The island is as wide as this line, so a field
 * that grew with each figure typed would move the whole island. */
const ISLAND_FIGURES = 6;

export function DimensionsField({
  height,
  isDisabled,
  onChange,
  width,
}: DimensionsFieldProps) {
  const [linked, setLinked] = useState(true);
  // In a window's rows the two fields are one width, lined up; on the island
  // each keeps room for a picture's largest side, so the line, and the
  // island sized to it, stay still while a size is typed.
  const isIsland = useSkin() === "island";
  const fieldWidth = isIsland ? undefined : "w-24";
  const reservedLength = isIsland ? ISLAND_FIGURES : undefined;
  // The size this field last asked for, so the answer coming back is not
  // taken as a new ratio to keep.
  const requestedRef = useRef<{ height: number; width: number } | null>(null);

  // Keep a stable ratio while linked, so consecutive edits don't drift.
  const lockedRatioRef = useRef<AspectRatioParts | undefined>(
    width > 0 && height > 0 ? reduceToRatio(width, height) : undefined,
  );
  const dimensionsRef = useRef({ height, width });
  dimensionsRef.current = { height, width };

  useLayoutEffect(() => {
    if (!linked || width <= 0 || height <= 0) return;

    const requested = requestedRef.current;
    requestedRef.current = null;
    if (requested && requested.width === width && requested.height === height)
      return;

    // Whole-pixel dimensions rarely land exactly on the locked ratio, so a
    // gesture that honours it would still redefine it a little every frame.
    // Dimensions already on the ratio leave it as it is.
    const locked = lockedRatioRef.current;
    if (locked && matchesRatio(width, height, locked)) return;

    // The size can begin with placeholder values while its document loads,
    // or change through reset or undo. Those outside values are the ratio to
    // keep, without letting this field's own linked edits redefine it on
    // every scrub step.
    lockedRatioRef.current = reduceToRatio(width, height);
  }, [height, linked, width]);

  useEffect(() => {
    if (!linked) {
      lockedRatioRef.current = undefined;
      return;
    }
    const dimensions = dimensionsRef.current;
    // Empty dimensions are not a ratio to keep: linking is often turned on by
    // a preset chosen before anything has been drawn, and the ratio that
    // preset just locked is the one to draw at.
    if (dimensions.width > 0 && dimensions.height > 0) {
      lockedRatioRef.current = reduceToRatio(
        dimensions.width,
        dimensions.height,
      );
    }
  }, [linked]);

  const change = (value: number, side: "height" | "width") => {
    const ratio = linked
      ? (lockedRatioRef.current ??
        (width > 0 && height > 0 ? reduceToRatio(width, height) : undefined))
      : undefined;
    if (!ratio || ratio.ratioWidth <= 0 || ratio.ratioHeight <= 0) {
      onChange({ [side]: value });
      return;
    }
    const next = dimensionsAtRatio(value, side, ratio);
    requestedRef.current = next;
    onChange(next);
  };

  return (
    <div className="flex items-center gap-inline">
      <NumberField
        aria-label={t("editor-panels-width")}
        className={fieldWidth}
        isDisabled={isDisabled}
        onChange={(value) => {
          change(value, "width");
        }}
        reservedLength={reservedLength}
        rightSection="px"
        value={width}
      />
      <IconToggleButton
        aria-label={t("editor-panels-keep-aspect-ratio")}
        isDisabled={isDisabled}
        isSelected={linked}
        off={<Unlink />}
        onChange={setLinked}
      >
        <Link />
      </IconToggleButton>
      <NumberField
        aria-label={t("editor-panels-height")}
        className={fieldWidth}
        isDisabled={isDisabled}
        onChange={(value) => {
          change(value, "height");
        }}
        reservedLength={reservedLength}
        rightSection="px"
        value={height}
      />
    </div>
  );
}
