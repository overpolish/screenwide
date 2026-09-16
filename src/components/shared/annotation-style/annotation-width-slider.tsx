// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Slider } from "../../base/slider/slider";

import {
  ANNOTATION_WIDTHS,
  annotationWidthAt,
  annotationWidthIndex,
} from "./widths";

/**
 * How heavy a mark is drawn.
 *
 * The strokes are not evenly spaced, so the knob runs over the preset's place
 * in the list; the ticks say how many there are. A width from an older
 * document lands the knob on the nearest preset rather than nowhere.
 */
export function AnnotationWidthSlider({
  isDisabled,
  onChange,
  value,
}: {
  onChange: (width: number) => void;
  /** Stroke width in output pixels. */
  value: number;
  isDisabled?: boolean;
}) {
  return (
    <Slider
      aria-label="Width"
      className="w-48"
      isDisabled={isDisabled}
      maxValue={ANNOTATION_WIDTHS.length - 1}
      minValue={0}
      onChange={(index) => {
        onChange(annotationWidthAt(index));
      }}
      showTicks
      step={1}
      value={annotationWidthIndex(value)}
    />
  );
}
