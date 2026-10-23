// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { Island, type IslandProps } from "../../../primitives/island/island";
import { IslandRow } from "../../../primitives/island/island-row";

import {
  AnnotateActions,
  AnnotateOptions,
  AnnotateToolButtons,
} from "./annotate-parts";
import {
  type AnnotatePartsProps,
  useAnnotateParts,
} from "./use-annotate-parts";

// The live overlay's tools on their own island: the tools to one side, and
// the colour and what can be done to what is drawn to the other; on a Mac
// with a notch, the notch between them. Under the tools hangs the pen of the
// one in hand, and the colours hang under the colour in its place.

export type AnnotateIslandProps = AnnotatePartsProps &
  Pick<IslandProps, "isShown" | "notch" | "onHidden" | "onShapeChange">;

export function AnnotateIsland({
  initialSection,
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
  const { hasColor, pen, showPen, showsColors, toggleColors } =
    useAnnotateParts({ initialSection, shape: settings.defaultShape });

  return (
    <Island
      anchor={showsColors ? "end" : "start"}
      aria-label={t("annotate-tools")}
      isShown={isShown}
      notch={notch}
      onHidden={onHidden}
      onShapeChange={onShapeChange}
      row={
        <IslandRow
          end={
            <AnnotateActions
              hasColor={hasColor}
              onClear={onClear}
              onDone={onDone}
              onToggleColors={toggleColors}
              onUndo={onUndo}
              settings={settings}
              showsColors={showsColors}
            />
          }
          start={
            <AnnotateToolButtons
              onChange={onChange}
              onPick={showPen}
              settings={settings}
            />
          }
        />
      }
      section={
        <AnnotateOptions
          onChange={onChange}
          pen={pen}
          savedColors={savedColors}
          settings={settings}
          showsColors={showsColors}
        />
      }
    />
  );
}
