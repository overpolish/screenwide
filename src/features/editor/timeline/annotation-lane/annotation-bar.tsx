// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Pin } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { Badge } from "../../../../primitives/badge/badge";
import { annotationToolLabel } from "../../../annotations/annotation-text";
import { ANNOTATION_TOOLS } from "../../tools/tool-registry";
import {
  StackBar,
  StackBarLine,
  type StackBarPlacement,
} from "../bar-stack/stack-bar";
import { type Span } from "../timeline-scale";

import {
  AnnotationBarPin,
  AnnotationBarUnsure,
  type TimelineAnnotationPin,
} from "./annotation-bar-pin";
import { placesAlong } from "./annotation-bar-waves";
import { AnnotationBarWavyLine } from "./annotation-bar-wavy-line";

import type { AnnotationKind } from "../../../../bindings/AnnotationKind";

// One annotation across the time it shows (`stack-bar.tsx`), the badge of
// the tool that drew it in its middle, a pin beside the tool's glyph where
// it follows its content (`annotation-bar-pin.tsx`), and its line a wave
// where it travels between its places in the scenes.

export type AnnotationBarProps = StackBarPlacement & {
  kind: AnnotationKind;
  /** Where its annotation travels between its places in the scenes, which
   * its line makes a wave over (`annotation-bar-wavy-line.tsx`). */
  glides?: Span[];
  /** Its pin, where it follows the content under it. */
  pin?: TimelineAnnotationPin;
};

export function AnnotationBar({
  glides,
  kind,
  pin,
  ...placement
}: AnnotationBarProps) {
  const { isSelected, scale, span, width } = placement;
  const tool = annotationToolLabel(kind);
  const waves = glides ? placesAlong(glides, { scale, span, width }) : [];
  return (
    <StackBar
      {...placement}
      badge={() => (
        <Badge
          className={cn(
            "px-0.5 text-surface transition-colors",
            isSelected
              ? "bg-primary text-on-primary"
              : "bg-fg-tertiary group-hover:bg-fg-secondary",
          )}
        >
          <ToolGlyph kind={kind} />
          {pin ? <Pin /> : null}
        </Badge>
      )}
      label={
        pin
          ? t("editor-timeline-annotation-pinned", { tool })
          : t("editor-timeline-annotation", { tool })
      }
      line={(look) => (
        <>
          {pin ? (
            <AnnotationBarPin
              isSelected={isSelected}
              mask={look.mask}
              pad={look.pad}
              pin={pin}
              scale={scale}
              span={span}
              width={width}
            />
          ) : null}
          {waves.length > 0 ? (
            <AnnotationBarWavyLine
              height={20}
              isSelected={isSelected}
              length={look.px}
              mask={look.mask}
              pad={look.pad}
              waves={waves}
            />
          ) : (
            <StackBarLine isSelected={isSelected} look={look} />
          )}
          {pin ? (
            <AnnotationBarUnsure
              isSelected={isSelected}
              mask={look.mask}
              pad={look.pad}
              pin={pin}
              scale={scale}
              span={span}
              width={width}
            />
          ) : null}
        </>
      )}
    />
  );
}

/** The glyph of the tool that drew an annotation, as the tool column has it. */
function ToolGlyph({ kind }: { kind: AnnotationKind }) {
  const Icon = ANNOTATION_TOOLS.find(({ id }) => id === kind)?.icon;
  return Icon ? <Icon /> : null;
}
