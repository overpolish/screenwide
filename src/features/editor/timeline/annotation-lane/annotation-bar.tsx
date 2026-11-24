// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Pin } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";

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
import { useFormatMarkMotion } from "../formats/format-mark-motion";
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

/** The differs dot's room: shut, and open as wide as what it holds. */
const COLLAPSED = { width: 0 };
const OPEN = { width: "auto" };

/** The air round the dot inside its room, in px: it sits midway between
 * the tool's glyph and the badge's rounded end, 5 px from each as drawn
 * (the glyph's box has 1 px of air of its own), since hard against the end
 * it read as crowded into the curve. */
const DOT_AIR = { paddingInlineEnd: 3, paddingInlineStart: 4 };

export type AnnotationBarProps = StackBarPlacement & {
  kind: AnnotationKind;
  /** Left out of the format in hand: the formats it shows in, named in a
   * list, which it is drawn faintly for; null where it shows here. */
  ghostOf?: string | null;
  // Its line makes a wave over them (`annotation-bar-wavy-line.tsx`).
  /** Where it glides between its places in the scenes. */
  glides?: Span[];
  /** Done differently in the format in hand: shown or laid out its own way. */
  isMarked?: boolean;
  /** What a right press on it offers a menu for, by id. */
  menuTarget?: string;
  /** Its pin, where it follows the content under it. */
  pin?: TimelineAnnotationPin;
};

export function AnnotationBar({
  ghostOf = null,
  glides,
  isMarked = false,
  kind,
  pin,
  ...placement
}: AnnotationBarProps) {
  const { isSelected, scale, span, width } = placement;
  const tool = annotationToolLabel(kind);
  const waves = glides ? placesAlong(glides, { scale, span, width }) : [];
  const markMotion = useFormatMarkMotion();
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
          <AnimatePresence initial={false}>
            {isMarked ? (
              <motion.span
                animate={OPEN}
                aria-hidden
                // The badge's gap before it is taken back for good, so all
                // the room there is is its own and the width alone opens
                // and closes it: a gap closed by an animated margin was
                // left over and went at once as the dot left the badge.
                className="-ms-inline flex shrink-0 items-center overflow-hidden"
                exit={COLLAPSED}
                initial={COLLAPSED}
                key="differs"
                transition={markMotion.transition}
              >
                <span className="flex shrink-0 items-center" style={DOT_AIR}>
                  <motion.span
                    className="size-1 rounded-full bg-current"
                    {...markMotion.mark()}
                  />
                </span>
              </motion.span>
            ) : null}
          </AnimatePresence>
        </Badge>
      )}
      isGhost={ghostOf !== null}
      keepsOnStrip
      label={
        ghostOf !== null
          ? t("editor-timeline-annotation-ghost", { formats: ghostOf, tool })
          : isMarked
            ? t("editor-timeline-annotation-differs", { tool })
            : pin
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
