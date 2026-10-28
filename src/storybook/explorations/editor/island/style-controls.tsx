// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type AnnotationHead } from "../../../../bindings/AnnotationHead";
import { ColorGrid } from "../../../../composites/color-grid/color-grid";
import { headChoice } from "../../../../features/annotations/annotation-choices";
import { ArrowHeadGlyph } from "../../../../features/annotations/options/arrow-head-glyph";
import { CornerChoice } from "../../../../features/annotations/options/corner-choice";
import { CounterPinIcon } from "../../../../features/annotations/options/counter-pin-icon";
import { FocusLookGlyph } from "../../../../features/annotations/options/focus-glyphs";
import { SizeDot } from "../../../../features/annotations/options/size-dot";
import {
  ANNOTATION_COUNTER_SIZES,
  ANNOTATION_WIDTHS,
} from "../../../../features/annotations/widths";
import { cn } from "../../../../lib/styling";
import { AngleDial } from "../../../../primitives/angle-dial/angle-dial";
import { IconButton } from "../../../../primitives/button/icon-button";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";
import { swatchEdge } from "../../../../primitives/swatch/swatch-styles";
import { type Pen } from "../../ideas/looks";
import { type AnnotationStyle } from "../model/annotation-style";
import { type Editor } from "../model/use-editor";

// How an annotation is drawn, laid out as the annotate island lays it out:
// what the pen draws with hangs under the row (an arrow's stroke and heads,
// a counter's disc and where its tail points, a focus's corners and what it
// does to the rest), and the colour in hand sits in the row, opening the
// colours in their place. Used for a pen's style, which new annotations
// take, and for one annotation's own.

/** The colour in hand, in the row: pressed, the colours hang in the pen's
 * place. A focus draws in no colour, so it has none. */
export function ColorButton({
  color,
  isOpen,
  onPress,
}: {
  color: string;
  isOpen: boolean;
  onPress: () => void;
}) {
  return (
    <IconButton aria-expanded={isOpen} aria-label="Colour" onPress={onPress}>
      <span
        className={cn("size-icon rounded-full", swatchEdge)}
        style={{ backgroundColor: color }}
      />
    </IconButton>
  );
}

/** The colours, as the annotate island offers them: ten to a row, scrolling
 * past five rows. */
export function ColorOptions({
  editor,
  onChange,
  value,
}: {
  editor: Editor;
  onChange: (color: string) => void;
  value: string;
}) {
  return (
    <ScrollArea
      className="p-inset-sm"
      constrainHeight
      rootClassName="-my-inset-sm max-h-34"
    >
      <ColorGrid
        columns={10}
        offersCustom
        onChange={onChange}
        onRemoveColor={editor.removeColor}
        onSaveColor={editor.saveColor}
        savedColors={editor.savedColors}
        value={value}
      />
    </ScrollArea>
  );
}

export function PenOptions({
  aimed,
  onChange,
  pen,
  style,
}: {
  /** Where a counter's tail points while it is not set, in degrees. */
  aimed: number;
  onChange: (change: Partial<AnnotationStyle>) => void;
  pen: Pen;
  style: AnnotationStyle;
}) {
  return (
    <div className="flex gap-separate">
      {pen === "arrow" ? (
        <>
          <PillGroup
            aria-label="Width"
            items={ANNOTATION_WIDTHS.map((points) => ({
              ariaLabel: `${String(points)} pt`,
              icon: <SizeDot diameter={points / 2 + 2} />,
              id: String(points),
            }))}
            onSelectionChange={(id) => {
              onChange({ width: Number(id) });
            }}
            selected={String(style.width)}
          />
          <PillGroup
            aria-label={headChoice.label()}
            items={headChoice.items().map((item) => ({
              ariaLabel: item.label ?? item.ariaLabel,
              icon: (
                <ArrowHeadGlyph
                  className="size-icon-lg"
                  head={item.id as AnnotationHead}
                />
              ),
              id: item.id,
            }))}
            onSelectionChange={(id) => {
              onChange({ head: headChoice.pick(id) });
            }}
            selected={headChoice.select(style.head)}
          />
        </>
      ) : pen === "counter" ? (
        <>
          <PillGroup
            aria-label="Size"
            items={ANNOTATION_COUNTER_SIZES.map((points) => ({
              ariaLabel: `${String(points)} pt`,
              icon: <SizeDot diameter={points / 4} />,
              id: String(points),
            }))}
            onSelectionChange={(id) => {
              onChange({ counterSize: Number(id) });
            }}
            selected={String(style.counterSize)}
          />
          <AngleDial
            aria-label="Where its tail points"
            isStepped
            onChange={(counterAngle) => {
              onChange({ counterAngle });
            }}
            value={style.counterAngle ?? aimed}
          >
            <CounterPinIcon className="size-icon" />
          </AngleDial>
        </>
      ) : (
        <>
          <CornerChoice
            onChange={(focusRadius) => {
              onChange({ focusRadius });
            }}
            value={style.focusRadius}
          />
          <PillGroup
            aria-label="Outside"
            items={(["blur", "dim"] as const).map((look) => ({
              ariaLabel: look === "blur" ? "Blur" : "Dim",
              icon: <FocusLookGlyph dims={look === "dim"} />,
              id: look,
            }))}
            onSelectionChange={(look) => {
              onChange({ focusDims: look === "dim" });
            }}
            selected={style.focusDims ? "dim" : "blur"}
          />
        </>
      )}
    </div>
  );
}
