// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnnotationAngleDial } from "../../components/shared/annotation-style/annotation-angle-dial";
import { AnnotationBlurGroup } from "../../components/shared/annotation-style/annotation-blur-group";
import { AnnotationEdgeGroup } from "../../components/shared/annotation-style/annotation-edge-group";
import { AnnotationFitGroup } from "../../components/shared/annotation-style/annotation-fit-group";
import { AnnotationHeadGroup } from "../../components/shared/annotation-style/annotation-head-group";
import { AnnotationInkGroup } from "../../components/shared/annotation-style/annotation-ink-group";
import { AnnotationRadiusField } from "../../components/shared/annotation-style/annotation-radius-field";
import { AnnotationStrokeGroup } from "../../components/shared/annotation-style/annotation-stroke-group";
import { AnnotationWidthSlider } from "../../components/shared/annotation-style/annotation-width-slider";
import { annotationSizes } from "../../components/shared/annotation-style/widths";
import { AnnotateSettings } from "../settings/types";

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
    <AnnotationWidthSlider
      label={settings.defaultShape === "counter" ? "Size" : "Width"}
      onChange={(size) => {
        onChange(
          settings.defaultShape === "counter"
            ? { defaultCounterSize: size }
            : { defaultWidth: size },
        );
      }}
      presets={annotationSizes(settings.defaultShape)}
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
          <AnnotationFitGroup
            onChange={(highlightManual) => {
              onChange({ highlightManual });
            }}
            value={settings.highlightManual}
          />
          <AnnotationInkGroup
            onChange={(highlightTint) => {
              onChange({ highlightTint });
            }}
            value={settings.highlightTint}
          />
          <AnnotationStrokeGroup
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
          <AnnotationStrokeGroup
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
          <AnnotationEdgeGroup
            onChange={(spotlightSoftness) => {
              onChange({ spotlightSoftness });
            }}
            value={settings.spotlightSoftness}
          />
          <AnnotationBlurGroup
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
          <AnnotationHeadGroup
            onChange={(defaultHead) => {
              onChange({ defaultHead });
            }}
            value={settings.defaultHead}
          />
        </>
      );
  }
}
