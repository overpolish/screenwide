// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import {
  generatePalette,
  generatePaletteFromLocked,
} from "../../../../../lib/palette-generator";

import { Background } from "./background";
import {
  backgroundGenerator,
  DEFAULT_GENERATOR_ID,
} from "./background-generators";
import {
  randomizeMeshBackground,
  randomMeshBackground,
  randomPaletteMode,
} from "./background-random";

type MeshBackground = Extract<Background, { kind: "mesh" }>;

const DEFAULT_SOLID = "#171717";

/**
 * The palette a generator reads, whatever the mesh arrived with.
 *
 * A generator takes three colours or four, and a mesh can change hands
 * between them, so a palette that is the wrong length is cut down, or filled
 * out from the generator's own colours, rather than shown at a length the
 * picture will not use.
 */
const generatorPalette = (colors: string[], generatorId: string) => {
  const { colorCount, defaultColors, id } = backgroundGenerator(generatorId);
  // The composition generator makes its own count: a blob per colour past the
  // first, and as many as it rolled. Its count is a floor rather than a rule,
  // so a mesh of five is still shown as five.
  const length =
    id === DEFAULT_GENERATOR_ID
      ? Math.max(colorCount, colors.length)
      : colorCount;
  return Array.from(
    { length },
    (_, index) => colors[index] ?? defaultColors[index],
  );
};

/**
 * A background built by hand, as any editor of it changes it: one colour, or
 * a mesh of several. The last of each is kept while the other is tried, so
 * switching back brings it back: a mesh its palette and painter, its
 * arrangement rolled again, as rolling it again does while it shows. A first
 * colour starts from the mesh's first.
 */
export function useBackgroundEdit({
  onChange,
  value,
}: {
  onChange: (background: Background) => void;
  value: Background;
}) {
  const [lastMesh, setLastMesh] = useState<MeshBackground | null>(
    value.kind === "mesh" ? value : null,
  );
  const [lastSolid, setLastSolid] = useState<string | null>(
    value.kind === "solid" ? value.color : null,
  );
  // Whatever is on the canvas is the last of its kind, however it got there.
  if (value.kind === "mesh" && lastMesh !== value) setLastMesh(value);
  if (value.kind === "solid" && lastSolid !== value.color)
    setLastSolid(value.color);
  const mesh = value.kind === "mesh" ? value : lastMesh;
  const palette = mesh ? generatorPalette(mesh.colors, mesh.generator) : [];
  const locked = mesh
    ? palette.map((_, index) => mesh.lockedColors[index] ?? false)
    : [];
  const changeMesh = (next: MeshBackground) => {
    setLastMesh(next);
    onChange(next);
  };
  return {
    kind: value.kind === "mesh" ? ("mesh" as const) : ("solid" as const),
    locked,
    mesh,
    palette,
    /** Makes the background again: a colour, or a mesh's colours not
     * locked, in a mood picked at random, the mesh laid out afresh. */
    randomise: () => {
      if (value.kind === "solid") {
        const [color] = generatePalette(randomPaletteMode(), 1);
        if (color) onChange({ color, kind: "solid" });
        return;
      }
      if (mesh)
        changeMesh(
          randomizeMeshBackground({
            ...mesh,
            colors: generatePaletteFromLocked({
              colors: palette,
              locked,
              mode: randomPaletteMode(),
            }),
          }),
        );
    },
    /** Rolls the mesh's arrangement again under its colours. */
    reroll: () => {
      if (mesh) changeMesh(randomizeMeshBackground(mesh));
    },
    setColors: (colors: string[]) => {
      if (mesh) changeMesh({ ...mesh, colors });
    },
    setKind: (kind: "mesh" | "solid") => {
      if (kind === "solid") {
        onChange({
          color: lastSolid ?? mesh?.colors[0] ?? DEFAULT_SOLID,
          kind: "solid",
        });
        return;
      }
      changeMesh(mesh ? randomizeMeshBackground(mesh) : randomMeshBackground());
    },
    setLocked: (lockedColors: boolean[]) => {
      if (mesh) changeMesh({ ...mesh, lockedColors });
    },
    setSolid: (color: string) => {
      onChange({ color, kind: "solid" });
    },
    solidColor: value.kind === "solid" ? value.color : DEFAULT_SOLID,
  };
}
