// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  type CSSProperties,
  type ReactNode,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { IslandFlow } from "../../../../primitives/island/island-flow";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";
import {
  TileChoice,
  type TileChoiceItem,
} from "../../../../primitives/tile-choice/tile-choice";
import {
  boxesDrawing,
  presetDrawing,
  type SceneDrawing,
  type ScenePreset,
} from "../../glyphs/scene-layout-drawing";
import { SceneLayoutGlyph } from "../../glyphs/scene-layout-glyph";
import { EDITOR_ISLAND_WIDTH } from "../editor-island-width";

import {
  layoutOptionsKind,
  sceneShape,
  type SceneFramingProps,
} from "./scene-framing";
import { SceneLayoutOptions } from "./scene-layout-options";
import { presetName } from "./scene-names";

// The layout group: how the scene lays out its panes, as tiles drawn large
// on the video's shape, and under them the choices of the one in use. The
// drawings tell the layouts apart, so the tiles carry their names only for
// assistive technology: a custom or saved layout has no name worth showing,
// and a long one would wrap its tile. A scene becomes custom when a pane is
// moved by hand on the picture, and is shown so here, drawn as placed; the
// presets follow, where the camera can be placed, then the layouts saved to
// use again. Past two and a half rows the tiles scroll, the half row saying
// there are more, so a long list of saved layouts never runs the island
// down over the picture.

const PRESETS: ScenePreset[] = [
  "split-two-thirds",
  "stacked",
  "picture-in-picture",
  "camera-only",
  "screen-only",
];

/** Rows of tiles shown before they scroll. */
const VISIBLE_ROWS = 2.5;

/** The tile grid's gap and the scrolled box's inset, in px: `gap-1` and
 * `p-1`. */
const TILE_GAP = 4;
const SCROLL_INSET = 4;

/** A layout's drawing as a tile's picture, its panes laid on the tile's
 * fill as on the frame. */
const tilePicture = (drawing: SceneDrawing) => (
  <SceneLayoutGlyph drawing={drawing} isFilling />
);

/** The tiles in a box that scrolls past two and a half rows, measured from
 * the first row, since a row's height follows the video's shape. The box
 * holds the tiles in a small inset, so a tile's focus ring at its edge is
 * not cut off, and reaches past them by as much on every side, so they keep
 * the group's width. A scroll area fills its parent, so it is widened by
 * both insets: a negative margin alone would only shift it towards the
 * start. The chosen tile is scrolled into view as the group opens. */
function ScrollingTiles({ children }: { children: ReactNode }) {
  const ref = useRef<HTMLDivElement>(null);
  const [maxHeight, setMaxHeight] = useState<number | null>(null);
  useLayoutEffect(() => {
    const tile = ref.current?.querySelector("button");
    if (!tile) return;
    const observer = new ResizeObserver(() => {
      setMaxHeight(
        SCROLL_INSET +
          tile.offsetHeight * VISIBLE_ROWS +
          TILE_GAP * Math.floor(VISIBLE_ROWS),
      );
    });
    observer.observe(tile);
    return () => {
      observer.disconnect();
    };
  }, []);
  return (
    <div
      ref={ref}
      style={
        maxHeight === null
          ? undefined
          : ({
              "--layout-tiles-height": `${String(maxHeight)}px`,
            } as CSSProperties)
      }
    >
      <ScrollArea
        className="p-1"
        constrainHeight
        onInitialized={(viewport) => {
          const chosen = viewport.querySelector("[data-selected]");
          if (!(chosen instanceof HTMLElement)) return;
          const box = viewport.getBoundingClientRect();
          const tile = chosen.getBoundingClientRect();
          if (tile.bottom > box.bottom) {
            viewport.scrollTop += tile.bottom - box.bottom + SCROLL_INSET;
          }
        }}
        rootClassName="-m-1 w-[calc(100%+0.5rem)] max-h-(--layout-tiles-height)"
      >
        {children}
      </ScrollArea>
    </div>
  );
}

export function SceneLayoutGroup(props: SceneFramingProps) {
  const { canPlaceCamera, layout, onLayoutChange, savedLayouts, variant } =
    props;
  const shape = sceneShape(props);
  const optionsKind = layoutOptionsKind(props);
  const chosen =
    layout.kind === "preset"
      ? layout.preset
      : layout.kind === "custom"
        ? "custom"
        : layout.id;
  const tiles: TileChoiceItem<string>[] = [
    ...(layout.kind === "custom"
      ? [
          {
            id: "custom",
            label: t("editor-island-scene-custom"),
            picture: tilePicture(boxesDrawing(layout.boxes, shape)),
          },
        ]
      : []),
    // Without the camera in the video, only the layouts that need none,
    // and the one the scene already has, so it still shows what it is.
    ...PRESETS.filter(
      (preset) =>
        canPlaceCamera || preset === "screen-only" || preset === chosen,
    ).map((preset) => ({
      id: preset,
      label: presetName(preset),
      picture: tilePicture(
        presetDrawing(preset, preset === chosen ? variant : {}, shape),
      ),
    })),
    // Numbered among all saved layouts, so a layout keeps its number
    // whether or not the others can be chosen.
    ...savedLayouts
      .map(({ boxes, id }, index) => ({
        hasCamera: boxes.camera !== undefined,
        id,
        label: t("editor-island-scene-saved-layout", { number: index + 1 }),
        picture: tilePicture(boxesDrawing(boxes, shape)),
      }))
      .filter(
        ({ hasCamera, id }) => canPlaceCamera || !hasCamera || id === chosen,
      )
      .map(({ id, label, picture }) => ({ id, label, picture })),
  ];
  return (
    // At least the width the editor island's parts share, which fits within
    // the narrowest scene row, so the row's title and buttons stay put as
    // the groups change under it, and across the whole section once it is
    // stretched to the island's box. The tiles and the options of the one
    // chosen stand apart; the options' own rows sit as close as the
    // island's parts do.
    <IslandFlow
      className={cn(
        "flex min-w-full flex-col gap-separate",
        EDITOR_ISLAND_WIDTH.part,
      )}
    >
      <ScrollingTiles>
        <TileChoice
          aria-label={t("editor-island-scene-layouts")}
          items={tiles}
          onChange={(id) => {
            if (id === "custom") return;
            onLayoutChange(
              (PRESETS as string[]).includes(id)
                ? { kind: "preset", preset: id as ScenePreset }
                : { id, kind: "saved" },
            );
          }}
          pictureAspect={shape.aspect}
          showsLabels={false}
          value={chosen}
        />
      </ScrollingTiles>
      {optionsKind === null ? null : (
        <SceneLayoutOptions key={optionsKind} {...props} />
      )}
    </IslandFlow>
  );
}
