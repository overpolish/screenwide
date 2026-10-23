// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ComponentType } from "react";

import { Eraser, Focus, Undo2, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useState } from "react";

import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { Island, type IslandProps } from "../../../primitives/island/island";
import { IslandButton } from "../../../primitives/island/island-controls";
import { focusIn } from "../../../primitives/island/island-motion";
import { IslandRow } from "../../../primitives/island/island-row";
import { swatchEdge } from "../../../primitives/swatch/swatch-styles";
import { annotationToolName } from "../../annotations/annotation-text";
import {
  ArrowToolIcon,
  CounterToolIcon,
  DrawToolIcon,
  HighlightToolIcon,
  ShapeToolIcon,
} from "../../annotations/annotation-tool-icons";

import { AnnotateColors } from "./annotate-colors";
import { ArrowPen } from "./arrow-pen";
import { CounterPen } from "./counter-pen";
import { DrawPen } from "./draw-pen";
import { FocusPen } from "./focus-pen";
import { HighlightPen } from "./highlight-pen";
import { ShapePen } from "./shape-pen";

import type { AnnotationKind } from "../../annotations/types";
import type { AnnotateToolbarProps } from "../annotate-toolbar";
import type { SectionProps } from "./section-props";

// The live overlay's tools on the island: the tools to one side, and the
// colour and what can be done to what is drawn to the other; on a Mac with a
// notch, the notch between them. Under the tools hangs the pen of the one in
// hand. The colour opens the colours in the pen's place; they stay open while
// colours are tried, and pressing the colour again, a tool or Escape steps
// back to the pen.

type Section = "colors" | "pen";

/** The tools the island offers, in the toolbar's order, each with its glyph
 * and the pen that hangs under the row while it is in hand.
 *
 * The spotlight is offered as Focus: one tool for drawing the eye to a box,
 * by blurring what lies around it or dimming it (`FocusPen`). It keeps the
 * spotlight's kind and settings until the overlay draws a focus of its own,
 * and is named and drawn here alone until the shared tool names and icons
 * follow. It draws in no colour, so the colour is not offered while it is in
 * hand. */
const ISLAND_TOOLS: {
  icon: typeof ArrowToolIcon;
  id: AnnotationKind;
  pen: ComponentType<SectionProps>;
  hasNoColor?: boolean;
  name?: () => string;
}[] = [
  { icon: ArrowToolIcon, id: "arrow", pen: ArrowPen },
  { icon: CounterToolIcon, id: "counter", pen: CounterPen },
  { icon: HighlightToolIcon, id: "highlight", pen: HighlightPen },
  { icon: ShapeToolIcon, id: "shape", pen: ShapePen },
  {
    hasNoColor: true,
    icon: Focus,
    id: "spotlight",
    name: () => t("annotate-island-focus-name"),
    pen: FocusPen,
  },
  { icon: DrawToolIcon, id: "draw", pen: DrawPen },
];

export type AnnotateIslandProps = Omit<AnnotateToolbarProps, "onSizeChange"> &
  Pick<IslandProps, "isShown" | "notch" | "onHidden" | "onShapeChange"> & {
    /** What hangs under the row when it first shows. */
    initialSection?: Section;
  };

export function AnnotateIsland({
  initialSection = "pen",
  isShown = true,
  notch,
  onChange,
  onClear,
  onDone,
  onHidden,
  onShapeChange,
  onUndo,
  savedColors,
  settings,
}: AnnotateIslandProps) {
  const [section, setSection] = useState<Section>(initialSection);
  const tool = ISLAND_TOOLS.find(({ id }) => id === settings.defaultShape);
  const Pen = tool?.pen;
  const hasColor = tool?.hasNoColor !== true;
  // A tool that draws in no colour hangs its own pen, even where the colours
  // were open for the tool before it.
  const showsColors = section === "colors" && hasColor;

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") setSection("pen");
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, []);

  const row = (
    <IslandRow
      end={
        <>
          {/* A tool that draws in no colour takes the colour out of the row:
              it is lifted out to blur away while the row closes up, and
              comes back into focus as the row opens for it. */}
          <AnimatePresence initial={false} mode="popLayout">
            {hasColor ? (
              <motion.span
                key="color"
                {...focusIn}
                className="flex"
                initial={focusIn.exit}
              >
                <IslandButton
                  aria-expanded={showsColors}
                  aria-label={t("annotation-color")}
                  isOpen={showsColors}
                  onPress={() => {
                    setSection((current) =>
                      current === "colors" ? "pen" : "colors",
                    );
                  }}
                >
                  <span
                    className={cn("size-icon rounded-full", swatchEdge)}
                    style={{ backgroundColor: settings.defaultColor }}
                  />
                </IslandButton>
              </motion.span>
            ) : null}
          </AnimatePresence>
          <IslandButton aria-label={t("annotate-undo-label")} onPress={onUndo}>
            <Undo2 aria-hidden className="size-icon rtl:-scale-x-100" />
          </IslandButton>
          <IslandButton
            aria-label={t("annotate-clear-label")}
            onPress={onClear}
          >
            <Eraser aria-hidden className="size-icon" />
          </IslandButton>
          <IslandButton aria-label={t("annotate-close-label")} onPress={onDone}>
            <X aria-hidden className="size-icon" />
          </IslandButton>
        </>
      }
      start={ISLAND_TOOLS.map(({ icon: Icon, id, name }) => (
        <IslandButton
          aria-label={name?.() ?? annotationToolName(id)}
          aria-pressed={id === settings.defaultShape}
          isOpen={id === settings.defaultShape}
          key={id}
          onPress={() => {
            if (id !== settings.defaultShape) onChange({ defaultShape: id });
            setSection("pen");
          }}
        >
          <Icon aria-hidden className="size-icon" />
        </IslandButton>
      ))}
    />
  );

  return (
    <Island
      anchor={showsColors ? "end" : "start"}
      aria-label={t("annotate-tools")}
      isShown={isShown}
      notch={notch}
      onHidden={onHidden}
      onShapeChange={onShapeChange}
      row={row}
      section={
        <AnimatePresence initial={false} mode="popLayout">
          {showsColors ? (
            <AnnotateColors
              key="colors"
              onChange={onChange}
              savedColors={savedColors}
              settings={settings}
            />
          ) : Pen ? (
            <Pen
              key={settings.defaultShape}
              onChange={onChange}
              settings={settings}
            />
          ) : null}
        </AnimatePresence>
      }
    />
  );
}
