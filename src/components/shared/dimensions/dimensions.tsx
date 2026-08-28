// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Link, RotateCcw, Unlink } from "lucide-react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";

import { cn } from "../../../lib/styling";
import { IconButton, IconToggleButton } from "../../base/button/icon-button";
import { FieldGroup } from "../../base/field-group/field-group";
import { NumberField } from "../../base/input-fields/number-field";

import {
  AspectRatioParts,
  dimensionsAtRatio,
  matchesRatio,
  reduceToRatio,
} from "./aspect-ratio-math";

const numberFieldStyles: React.ComponentProps<typeof NumberField> = {
  className: "w-20",
  rightSection: "px",
  showSteppers: false,
  size: "compact",
};

type DimensionsProps = {
  className?: string;
  defaultHeight?: number;
  defaultWidth?: number;
  height?: number;
  initialLinked?: boolean;
  label?: string;
  layout?: "inline" | "stacked";
  onRatioChange?: (ratio: number | undefined) => void;
  onReset?: () => void;
  setDimensions?: (width: number, height: number) => void;
  setHeight?: (value: number) => void;
  setWidth?: (value: number) => void;
  width?: number;
};

export const Dimensions = ({
  className,
  defaultHeight = 1080,
  defaultWidth = 1920,
  height,
  initialLinked = false,
  label = "Dimensions",
  layout = "inline",
  onRatioChange,
  onReset,
  setDimensions,
  setHeight,
  setWidth,
  width,
}: DimensionsProps) => {
  // Determine if fully controlled (both values and setters provided)
  const isControlled =
    width != null && height != null && setWidth != null && setHeight != null;

  // Uncontrolled internal state
  const [uWidth, setUWidth] = useState<number>(width ?? defaultWidth);
  const [uHeight, setUHeight] = useState<number>(height ?? defaultHeight);
  const requestedDimensionsRef = useRef<{
    height: number;
    width: number;
  } | null>(null);

  // Resolved values and setters
  const widthValue = isControlled ? width : uWidth;
  const heightValue = isControlled ? height : uHeight;
  const setWidthValue = (value: number) => {
    (setWidth ?? setUWidth)(value);
  };
  const setHeightValue = (value: number) => {
    (setHeight ?? setUHeight)(value);
  };
  const setDimensionValues = (newWidth: number, newHeight: number) => {
    if (setDimensions) {
      requestedDimensionsRef.current = {
        height: newHeight,
        width: newWidth,
      };
      setDimensions(newWidth, newHeight);
      return;
    }
    setWidthValue(newWidth);
    setHeightValue(newHeight);
  };

  const [linked, setLinked] = useState(initialLinked);

  // Keep a stable ratio while linked, so consecutive edits don't drift.
  const lockedRatioRef = useRef<AspectRatioParts | undefined>(
    initialLinked && widthValue > 0 && heightValue > 0
      ? reduceToRatio(widthValue, heightValue)
      : undefined,
  );
  const dimensionsRef = useRef({ height: heightValue, width: widthValue });
  dimensionsRef.current = { height: heightValue, width: widthValue };

  useLayoutEffect(() => {
    if (!linked || widthValue <= 0 || heightValue <= 0) return;

    const requested = requestedDimensionsRef.current;
    requestedDimensionsRef.current = null;
    if (
      requested &&
      requested.width === widthValue &&
      requested.height === heightValue
    )
      return;

    // Whole-pixel dimensions rarely land exactly on the locked ratio, so a
    // gesture that honours it would still redefine it a little every frame.
    // Dimensions already on the ratio leave it as it is.
    const locked = lockedRatioRef.current;
    if (locked && matchesRatio(widthValue, heightValue, locked)) return;

    // Controlled dimensions can begin with temporary placeholder values while
    // their artifact loads, or change through reset/undo. Treat those external
    // values as the ratio to preserve without letting our own linked edits
    // redefine it on every scrub step.
    lockedRatioRef.current = reduceToRatio(widthValue, heightValue);
  }, [heightValue, linked, widthValue]);

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

  const getLockedRatio = (): AspectRatioParts | undefined =>
    lockedRatioRef.current ??
    (widthValue > 0 && heightValue > 0
      ? reduceToRatio(widthValue, heightValue)
      : undefined);

  const adjustToRatio = (
    value: number,
    editingDimension: "width" | "height",
    ratio: AspectRatioParts | undefined,
  ) => {
    // If no ratio provided, just apply the direct edit
    if (!ratio) {
      if (editingDimension === "width") setWidthValue(value);
      else setHeightValue(value);
      return;
    }

    const { ratioHeight, ratioWidth } = ratio;

    if (ratioWidth <= 0 || ratioHeight <= 0) {
      if (editingDimension === "width") setWidthValue(value);
      else setHeightValue(value);
      return;
    }

    const { height: newHeight, width: newWidth } = dimensionsAtRatio(
      value,
      editingDimension,
      ratio,
    );

    setDimensionValues(newWidth, newHeight);
  };

  const onChangeWidth = (value: number) => {
    if (linked) {
      adjustToRatio(value, "width", getLockedRatio());
      return;
    }
    setWidthValue(value);
  };

  const onChangeHeight = (value: number) => {
    if (linked) {
      adjustToRatio(value, "height", getLockedRatio());
      return;
    }
    setHeightValue(value);
  };

  // Notify parent of the active aspect ratio to enforce in external resizers (e.g. RND)
  const lastRatioSentRef = useRef<number | undefined>(undefined);
  const onRatioChangeRef = useRef(onRatioChange);

  useEffect(() => {
    onRatioChangeRef.current = onRatioChange;
  }, [onRatioChange]);

  useEffect(() => {
    let ratioNum: number | undefined = undefined;
    if (linked) {
      const r =
        lockedRatioRef.current ??
        (widthValue > 0 && heightValue > 0
          ? reduceToRatio(widthValue, heightValue)
          : undefined);
      if (r && r.ratioHeight > 0) {
        ratioNum = r.ratioWidth / r.ratioHeight;
      }
    }

    if (onRatioChangeRef.current && lastRatioSentRef.current !== ratioNum) {
      onRatioChangeRef.current(ratioNum);
      lastRatioSentRef.current = ratioNum;
    } else {
      lastRatioSentRef.current = ratioNum;
    }
  }, [linked, widthValue, heightValue]);

  const dimensionFields = (
    <div className="flex flex-row items-center">
      {onReset ? (
        <IconButton
          aria-label="Reset dimensions"
          onPress={onReset}
          size="compact"
        >
          <RotateCcw />
        </IconButton>
      ) : null}
      <FieldGroup className="flex items-center">
        <NumberField
          {...numberFieldStyles}
          aria-label="Aspect Ratio Width"
          onChange={onChangeWidth}
          value={widthValue}
        />

        <IconToggleButton
          aria-label={linked ? "Unlink dimensions" : "Link dimensions"}
          isSelected={linked}
          off={<Unlink />}
          onChange={(isSelected) => {
            setLinked(isSelected);
          }}
          size="compact"
        >
          <Link />
        </IconToggleButton>

        <NumberField
          {...numberFieldStyles}
          aria-label="Aspect Ratio Height"
          onChange={onChangeHeight}
          value={heightValue}
        />
      </FieldGroup>
    </div>
  );
  return (
    <div
      className={cn(
        layout === "stacked"
          ? "gap-section flex w-full flex-col"
          : "gap-control flex flex-row items-center",
        className,
      )}
    >
      {layout === "stacked" ? (
        <div className="flex items-center">
          <span className="text-xs text-content-fg">{label}</span>
          <div className="grow" />
          {dimensionFields}
        </div>
      ) : (
        dimensionFields
      )}
    </div>
  );
};
