// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Slider } from "../../base/slider/slider";

import {
  ANNOTATION_WIDTHS,
  annotationWidthAt,
  annotationWidthIndex,
} from "./widths";

/**
 * How heavy an annotation is drawn: an arrow's stroke, or a counter's disc.
 *
 * The sizes are not evenly spaced, so the knob runs over the preset's place
 * in the list; the ticks say how many there are. A size from an older
 * document lands the knob on the nearest preset rather than nowhere.
 */
export function AnnotationWidthSlider({
  isDisabled,
  label = "Width",
  onChange,
  presets = ANNOTATION_WIDTHS,
  value,
}: {
  onChange: (width: number) => void;
  /** The size in output pixels: an arrow's stroke or a counter's diameter. */
  value: number;
  isDisabled?: boolean;
  label?: string;
  presets?: number[];
}) {
  return (
    <Slider
      aria-label={label}
      className="w-48"
      isDisabled={isDisabled}
      maxValue={presets.length - 1}
      minValue={0}
      onChange={(index) => {
        onChange(annotationWidthAt(index, presets));
      }}
      showTicks
      step={1}
      value={annotationWidthIndex(value, presets)}
    />
  );
}
