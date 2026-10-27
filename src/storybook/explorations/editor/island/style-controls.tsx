// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type AnnotationHead } from "../../../../bindings/AnnotationHead";
import { ColorGrid } from "../../../../composites/color-grid/color-grid";
import { ArrowHeadGlyph } from "../../../../features/annotate/island/arrow-head-glyph";
import { CornerChoice } from "../../../../features/annotate/island/corner-choice";
import { CounterPinIcon } from "../../../../features/annotate/island/counter-pin-icon";
import { FocusLookGlyph } from "../../../../features/annotate/island/focus-glyphs";
import { SizeDot } from "../../../../features/annotate/island/size-dot";
import { headChoice } from "../../../../features/annotations/annotation-choices";
import {
  ANNOTATION_COUNTER_SIZES,
  ANNOTATION_WIDTHS,
} from "../../../../features/annotations/widths";
import { cn } from "../../../../lib/styling";
import { IslandChoice } from "../../../../primitives/island/island-choice";
import { IslandButton } from "../../../../primitives/island/island-controls";
import { IslandDial } from "../../../../primitives/island/island-dial";
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
    <IslandButton
      aria-expanded={isOpen}
      aria-label="Colour"
      isOpen={isOpen}
      onPress={onPress}
    >
      <span
        className={cn("size-icon rounded-full", swatchEdge)}
        style={{ backgroundColor: color }}
      />
    </IslandButton>
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
          <IslandChoice
            items={ANNOTATION_WIDTHS.map((points) => ({
              content: <SizeDot diameter={points / 2 + 2} />,
              id: String(points),
              label: `${String(points)} pt`,
            }))}
            label="Width"
            onChange={(id) => {
              onChange({ width: Number(id) });
            }}
            value={String(style.width)}
          />
          <IslandChoice
            items={headChoice.items().map((item) => ({
              content: (
                <ArrowHeadGlyph
                  className="size-icon-lg"
                  head={item.id as AnnotationHead}
                />
              ),
              id: item.id,
              label: item.label ?? item.ariaLabel,
            }))}
            label={headChoice.label()}
            onChange={(id) => {
              onChange({ head: headChoice.pick(id) });
            }}
            value={headChoice.select(style.head)}
          />
        </>
      ) : pen === "counter" ? (
        <>
          <IslandChoice
            items={ANNOTATION_COUNTER_SIZES.map((points) => ({
              content: <SizeDot diameter={points / 4} />,
              id: String(points),
              label: `${String(points)} pt`,
            }))}
            label="Size"
            onChange={(id) => {
              onChange({ counterSize: Number(id) });
            }}
            value={String(style.counterSize)}
          />
          <IslandDial
            aria-label="Where its tail points"
            onChange={(counterAngle) => {
              onChange({ counterAngle });
            }}
            value={style.counterAngle ?? aimed}
          >
            <CounterPinIcon className="size-icon" />
          </IslandDial>
        </>
      ) : (
        <>
          <CornerChoice
            onChange={(focusRadius) => {
              onChange({ focusRadius });
            }}
            value={style.focusRadius}
          />
          <IslandChoice
            items={(["blur", "dim"] as const).map((look) => ({
              content: <FocusLookGlyph look={look} />,
              id: look,
              label: look === "blur" ? "Blur" : "Dim",
            }))}
            label="Outside"
            onChange={(look) => {
              onChange({ focusDims: look === "dim" });
            }}
            value={style.focusDims ? "dim" : "blur"}
          />
        </>
      )}
    </div>
  );
}
