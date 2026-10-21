// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Slider } from "../../primitives/slider/slider";
import { AnnotationAngleDial } from "../annotations/annotation-angle-dial";
import { AnnotationChoice } from "../annotations/annotation-choice";
import {
  blurChoice,
  edgeChoice,
  fitChoice,
  headChoice,
  inkChoice,
  strokeChoice,
} from "../annotations/annotation-choices";
import { AnnotationRadiusField } from "../annotations/annotation-radius-field";
import { annotationSizeLabel } from "../annotations/annotation-text";
import { annotationSizes } from "../annotations/widths";

import type { AnnotateSettings } from "../../bindings/AnnotateSettings";

/**
 * What the tool in hand is drawn with, beside the colour: each tool carries
 * the controls its annotation has. A live annotation cannot be picked up
 * again, so everything is chosen before it is drawn - a counter is aimed
 * before it is dropped rather than turned afterwards, a shape drawn by hand
 * and rounded, a spotlight rounded, faded and blurred.
 */
export function AnnotateToolControls({
  onChange,
  settings,
}: {
  onChange: (patch: Partial<AnnotateSettings>, immediate?: boolean) => void;
  settings: AnnotateSettings;
}) {
  const width = (
    <Slider
      aria-label={annotationSizeLabel(
        settings.defaultShape === "counter" ? "size" : "width",
      )}
      className="w-48"
      onChange={(size) => {
        onChange(
          settings.defaultShape === "counter"
            ? { defaultCounterSize: size }
            : { defaultWidth: size },
        );
      }}
      steps={annotationSizes(settings.defaultShape)}
      value={
        settings.defaultShape === "counter"
          ? settings.defaultCounterSize
          : settings.defaultWidth
      }
    />
  );
  switch (settings.defaultShape) {
    // A highlight fits the lines it covers, so its height is theirs.
    case "highlight":
      return (
        <>
          <AnnotationChoice
            choice={fitChoice}
            onChange={(highlightManual) => {
              onChange({ highlightManual });
            }}
            value={settings.highlightManual}
          />
          <AnnotationChoice
            choice={inkChoice}
            onChange={(highlightTint) => {
              onChange({ highlightTint });
            }}
            value={settings.highlightTint}
          />
          <AnnotationChoice
            choice={strokeChoice}
            onChange={(highlightHandDrawn) => {
              onChange({ highlightHandDrawn });
            }}
            value={settings.highlightHandDrawn}
          />
        </>
      );
    case "counter":
      return (
        <>
          {width}
          <AnnotationAngleDial
            onChange={(defaultCounterAngle, typed) => {
              onChange({ defaultCounterAngle }, typed);
            }}
            value={settings.defaultCounterAngle}
          />
        </>
      );
    case "shape":
      return (
        <>
          {width}
          <AnnotationChoice
            choice={strokeChoice}
            onChange={(shapeHandDrawn) => {
              onChange({ shapeHandDrawn });
            }}
            value={settings.shapeHandDrawn}
          />
          <AnnotationRadiusField
            onChange={(shapeRadius) => {
              onChange({ shapeRadius });
            }}
            value={settings.shapeRadius}
          />
        </>
      );
    // A spotlight draws no stroke and has no colour: its shade is fixed.
    case "spotlight":
      return (
        <>
          <AnnotationRadiusField
            onChange={(spotlightRadius) => {
              onChange({ spotlightRadius });
            }}
            value={settings.spotlightRadius}
          />
          <AnnotationChoice
            choice={edgeChoice}
            onChange={(spotlightSoftness) => {
              onChange({ spotlightSoftness });
            }}
            value={settings.spotlightSoftness}
          />
          <AnnotationChoice
            choice={blurChoice}
            onChange={(spotlightBlur) => {
              onChange({ spotlightBlur });
            }}
            value={settings.spotlightBlur}
          />
        </>
      );
    // A stroke has only its pen to choose.
    case "draw":
      return width;
    // The arrow, and the kinds the overlay never draws, which its settings
    // refuse.
    case "arrow":
    case "magnify":
    case "redact":
    case "text":
      return (
        <>
          {width}
          <AnnotationChoice
            choice={headChoice}
            onChange={(defaultHead) => {
              onChange({ defaultHead });
            }}
            value={settings.defaultHead}
          />
        </>
      );
  }
}
