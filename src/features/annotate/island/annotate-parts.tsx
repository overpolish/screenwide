// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ComponentType } from "react";

import { Eraser, Undo2, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";

import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import {
  IconButton,
  IconToggleButton,
} from "../../../primitives/button/icon-button";
import { focusIn } from "../../../primitives/island/island-motion";
import { swatchEdge } from "../../../primitives/swatch/swatch-styles";
import { annotationToolName } from "../../annotations/annotation-text";

import { AnnotateColors } from "./annotate-colors";
import { ISLAND_TOOLS } from "./island-tools";

import type { SectionProps } from "./section-props";
import type { AnnotatePartsProps } from "./use-annotate-parts";
import type { AnnotateSettings } from "../../../bindings/AnnotateSettings";

// The parts the annotate tools are made of, laid out by each island in its
// own way: the tools, what can be done besides drawing, and what hangs under
// them. The annotate island splits the first two round the notch; the
// recording island lines them up under its own row.

/** The tools, the one in hand chosen. Picking one steps back to its pen. */
export function AnnotateToolButtons({
  onChange,
  onPick,
  settings,
}: {
  onChange: AnnotatePartsProps["onChange"];
  onPick: () => void;
  settings: AnnotateSettings;
}) {
  return ISLAND_TOOLS.map(({ icon: Icon, id, name }) => (
    <IconToggleButton
      aria-label={name?.() ?? annotationToolName(id)}
      isSelected={id === settings.defaultShape}
      key={id}
      onChange={() => {
        if (id !== settings.defaultShape) onChange({ defaultShape: id });
        onPick();
      }}
    >
      <Icon aria-hidden className="size-icon" />
    </IconToggleButton>
  ));
}

/** The colour in hand, which opens the colours, then undo, clear and close.
 * A tool that draws in no colour takes the colour away: it is lifted out to
 * blur off while the island closes up, and comes back into focus as it
 * opens for it. */
export function AnnotateActions({
  hasColor,
  onClear,
  onDone,
  onToggleColors,
  onUndo,
  settings,
  showsColors,
}: Pick<AnnotatePartsProps, "onClear" | "onDone" | "onUndo" | "settings"> & {
  hasColor: boolean;
  onToggleColors: () => void;
  showsColors: boolean;
}) {
  return (
    <>
      <AnimatePresence initial={false} mode="popLayout">
        {hasColor ? (
          <motion.span
            key="color"
            {...focusIn}
            className="flex"
            initial={focusIn.exit}
          >
            <IconButton
              aria-expanded={showsColors}
              aria-label={t("annotation-color")}
              onPress={onToggleColors}
              variant="ghost"
            >
              <span
                className={cn("size-icon rounded-full", swatchEdge)}
                style={{ backgroundColor: settings.defaultColor }}
              />
            </IconButton>
          </motion.span>
        ) : null}
      </AnimatePresence>
      <IconButton
        aria-label={t("annotate-undo-label")}
        onPress={onUndo}
        variant="ghost"
      >
        <Undo2 aria-hidden className="size-icon rtl:-scale-x-100" />
      </IconButton>
      <IconButton
        aria-label={t("annotate-clear-label")}
        onPress={onClear}
        variant="ghost"
      >
        <Eraser aria-hidden className="size-icon" />
      </IconButton>
      <IconButton
        aria-label={t("annotate-close-label")}
        onPress={onDone}
        variant="ghost"
      >
        <X aria-hidden className="size-icon" />
      </IconButton>
    </>
  );
}

/** What hangs under the tools: the colours, or the pen of the tool in hand,
 * swapped as the island swaps its parts. */
export function AnnotateOptions({
  onChange,
  pen: Pen,
  savedColors,
  settings,
  showsColors,
}: Pick<AnnotatePartsProps, "onChange" | "savedColors" | "settings"> & {
  pen: ComponentType<SectionProps> | undefined;
  showsColors: boolean;
}) {
  return (
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
  );
}
