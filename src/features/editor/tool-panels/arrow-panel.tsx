// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrowLeftRight, Plus } from "lucide-react";
import { ToggleButtonGroup } from "react-aria-components";

import { Button } from "../../../components/base/button/button";
import {
  useColorPanel,
  usesSystemColorPanel,
} from "../../../components/base/input-fields/use-color-panel";
import { PillGroup } from "../../../components/base/pill-group/pill-group";
import { Slider } from "../../../components/base/slider/slider";
import { Switch } from "../../../components/base/switch/switch";
import { Text } from "../../../components/base/text/text";
import { BackgroundTile } from "../../../components/shared/background-picker/background-tile";
import {
  ANNOTATION_SWATCHES,
  isAnnotationSwatch,
  sameAnnotationColor,
} from "../annotation-palette";
import {
  ANNOTATION_WIDTHS,
  AnnotationHead,
  annotationWidthAt,
  annotationWidthIndex,
} from "../annotations";
import { EditorKind } from "../types";

import { useAnnotationColorMenu } from "./use-annotation-color-menu";
import { useToolPanelSnapshot } from "./use-tool-panel-snapshot";

const HEADS: { id: AnnotationHead; label: string }[] = [
  { id: "none", label: "None" },
  { id: "end", label: "End" },
  { id: "both", label: "Both" },
];

/**
 * The chosen mark's own controls: how heavy it is drawn, whether it draws
 * itself in over its clip, which ends carry a head, and what colour it is.
 * Animating takes a clip to animate over, so that row is the recording
 * editor's alone.
 *
 * This panel belongs to the mark rather than to a tool. It comes up the
 * moment an arrow is chosen, in either tool that can choose one, and goes
 * away when the choice does. Every change is committed through the same path
 * the drag on the picture uses, so it lands in the edit history as one edit,
 * and becomes the dress the next arrow is drawn in.
 *
 * The colours are the background picker's grid one row down: the palette,
 * then the colours of your own, then the Custom tile that opens the system
 * Colours panel. A colour the palette does not hold is kept the moment the
 * panel closes on it, and forgotten from the same right-click menu a saved
 * background is.
 */
export function ArrowPanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const { annotation, annotationColors, isSaving } = snapshot;
  const systemPanel = usesSystemColorPanel();
  const showColorMenu = useAnnotationColorMenu((color) => {
    change({ removeAnnotationColor: color });
  });
  // The panel reports every step of a drag; only what it closed on is kept,
  // so a sweep through the spectrum does not leave a dozen swatches behind.
  // The keep travels with the colour rather than after it, so the swatch the
  // panel settled on is shown from the moment it is chosen.
  const colorPanel = useColorPanel({
    onChange: (next) => {
      change({ annotationStyle: { color: next } });
    },
    onClose: (settled) => {
      if (settled)
        change({
          annotationStyle: { color: settled },
          saveAnnotationColor: settled,
        });
    },
  });

  if (!annotation) return <Text variant="body">Nothing selected</Text>;

  const { color, head, width } = annotation.style;
  const saved = annotationColors.filter((kept) => !isAnnotationSwatch(kept));
  const chosen =
    [...ANNOTATION_SWATCHES.map((swatch) => swatch.color), ...saved].find(
      (offered) => sameAnnotationColor(offered, color),
    ) ?? "custom";
  const applyColor = (next: string) => {
    change({ annotationStyle: { color: next } });
  };

  return (
    <div className="flex flex-col gap-section">
      <div className="flex items-center justify-between gap-section">
        <span className="text-body text-content-fg">Width</span>
        {/* The strokes are not evenly spaced, so the knob runs over the
            preset's place in the list; the ticks say how many there are and
            the picture says what each one looks like. */}
        <Slider
          aria-label="Width"
          className="w-48"
          isDisabled={isSaving}
          maxValue={ANNOTATION_WIDTHS.length - 1}
          minValue={0}
          onChange={(index) => {
            change({ annotationStyle: { width: annotationWidthAt(index) } });
          }}
          showTicks
          step={1}
          value={annotationWidthIndex(width)}
        />
      </div>

      {/* Drawing in and out happens over a clip, and only a recording has
          one: a screenshot is one instant, so there is no time for a mark to
          arrive over and the row is not offered there at all. */}
      {workspace === "recording" ? (
        <div className="flex items-center justify-between gap-section">
          <span className="text-body text-content-fg">Animate</span>
          <Switch
            aria-label="Animate"
            isDisabled={isSaving}
            isSelected={annotation.animated}
            onChange={(next) => {
              change({ annotationAnimated: next });
            }}
          />
        </div>
      ) : null}

      <div className="flex items-center justify-between gap-section">
        <span className="text-body text-content-fg">Head</span>
        <PillGroup
          aria-label="Head"
          display="label"
          isDisabled={isSaving}
          items={HEADS}
          onSelectionChange={(id) => {
            change({ annotationStyle: { head: id as AnnotationHead } });
          }}
          selected={head}
        />
      </div>

      {/* The head rides the end point, so turning the mark round points it
          the other way without redrawing it: the same commit path the drag
          on the picture uses. */}
      <div className="flex items-center justify-between gap-section">
        <span className="text-body text-content-fg">Direction</span>
        <Button
          aria-label="Reverse the arrow"
          isDisabled={isSaving}
          onPress={() => {
            change({ reverseAnnotation: true });
          }}
        >
          <ArrowLeftRight aria-hidden="true" />
          Reverse
        </Button>
      </div>

      {/* The swatches say what they are, so the row carries no heading; it
          keeps the section gap its two labelled neighbours sit on. */}
      <ToggleButtonGroup
        aria-label="Colour"
        className="flex flex-wrap justify-center gap-control"
        disallowEmptySelection
        isDisabled={isSaving}
        selectedKeys={new Set([chosen])}
        selectionMode="single"
      >
        {ANNOTATION_SWATCHES.map((swatch) => (
          <BackgroundTile
            ariaLabel={swatch.name}
            background={{ color: swatch.color, kind: "solid" }}
            id={swatch.color}
            isDisabled={isSaving}
            isSelected={chosen === swatch.color}
            key={swatch.color}
            onPress={() => {
              applyColor(swatch.color);
            }}
          />
        ))}
        {saved.map((kept) => (
          <BackgroundTile
            ariaLabel={kept}
            background={{ color: kept, kind: "solid" }}
            id={kept}
            isDisabled={isSaving}
            isSelected={chosen === kept}
            key={kept}
            onContextMenu={(event) => {
              event.preventDefault();
              if (isSaving) return;
              void showColorMenu(
                kept,
                event.currentTarget.getBoundingClientRect(),
              );
            }}
            onPress={() => {
              applyColor(kept);
            }}
          />
        ))}
        {/* The Custom tile is a button that opens the system Colours panel;
            a colour chosen there becomes a tile of its own, so this one is
            the glyph and is chosen only while none of the tiles matches.
            Off the Mac there is no panel to open, so the platform's own
            colour input sits over the tile at zero opacity, exactly as the
            colour well does, and settles on blur rather than on every step
            of a drag. */}
        <span className="relative inline-flex">
          <BackgroundTile
            ariaLabel="Custom colour"
            id="custom"
            isDisabled={isSaving}
            isSelected={chosen === "custom"}
            onPress={() => {
              if (systemPanel) colorPanel.open(color);
            }}
          >
            <Plus aria-hidden="true" />
          </BackgroundTile>
          {systemPanel ? null : (
            <input
              aria-label="Custom colour"
              className="absolute inset-0 size-full cursor-default opacity-0 outline-none"
              disabled={isSaving}
              onBlur={() => {
                change({
                  annotationStyle: { color },
                  saveAnnotationColor: color,
                });
              }}
              onChange={(event) => {
                applyColor(event.currentTarget.value.toLowerCase());
              }}
              type="color"
              value={color}
            />
          )}
        </span>
      </ToggleButtonGroup>
    </div>
  );
}
