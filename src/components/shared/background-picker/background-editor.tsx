// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { Button } from "../../base/button/button";
import { ColorPaletteGenerator } from "../../base/input-fields/color-palette-generator";
import { ColorSwatch } from "../../base/input-fields/color-swatch";
import { PillGroup } from "../../base/pill-group/pill-group";

import { Background } from "./background";
import {
  backgroundGenerator,
  DEFAULT_GENERATOR_ID,
} from "./background-generators";
import {
  randomMeshBackground,
  randomizeMeshBackground,
} from "./background-random";

const backgroundKinds = [
  { id: "solid", label: "Solid" },
  { id: "mesh", label: "Mesh" },
];

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
 * A background built by hand, and saved under a name if it is worth keeping.
 *
 * Returning to Mesh preserves its palette and generator while rerolling the
 * arrangement, just like pressing Mesh when it is already selected.
 */
export function BackgroundEditor({
  isDisabled,
  onChange,
  onSavePreset,
  value,
}: {
  onChange: (background: Background) => void;
  onSavePreset: (background: Background) => void;
  value: Background;
  isDisabled?: boolean;
}) {
  const [lastMesh, setLastMesh] = useState<Extract<
    Background,
    { kind: "mesh" }
  > | null>(value.kind === "mesh" ? value : null);
  const solidColor = value.kind === "solid" ? value.color : DEFAULT_SOLID;
  const mesh = value.kind === "mesh" ? value : lastMesh;
  const palette = mesh ? generatorPalette(mesh.colors, mesh.generator) : [];
  const kind = value.kind === "mesh" ? "mesh" : "solid";

  const changeMesh = (next: Extract<Background, { kind: "mesh" }>) => {
    setLastMesh(next);
    onChange(next);
  };

  return (
    <div className="flex flex-col gap-control-inset">
      <div className="flex items-center justify-between gap-section">
        <span className="text-body text-content-fg">Type</span>
        <PillGroup
          aria-label="Background type"
          display="label"
          isDisabled={isDisabled}
          items={backgroundKinds.map((item) =>
            item.id === "mesh"
              ? {
                  ...item,
                  onPress: () => {
                    if (kind === "mesh" && mesh) {
                      changeMesh(randomizeMeshBackground(mesh));
                    }
                  },
                }
              : item,
          )}
          onSelectionChange={(nextKind) => {
            if (nextKind === "solid") {
              onChange({ color: solidColor, kind: "solid" });
              return;
            }
            changeMesh(
              mesh ? randomizeMeshBackground(mesh) : randomMeshBackground(),
            );
          }}
          selected={kind}
        />
      </div>

      {kind === "solid" || !mesh ? (
        <div className="flex items-center justify-between gap-section">
          <span className="text-body text-content-fg">Colour</span>
          <ColorSwatch
            ariaLabel="Background colour"
            isDisabled={isDisabled}
            onChange={(color) => {
              onChange({ color, kind: "solid" });
            }}
            value={solidColor}
          />
        </div>
      ) : (
        <ColorPaletteGenerator
          colors={palette}
          isDisabled={isDisabled}
          locked={palette.map((_, index) => mesh.lockedColors[index] ?? false)}
          onChange={(colors) => {
            changeMesh({ ...mesh, colors });
          }}
          onLockedChange={(lockedColors) => {
            changeMesh({ ...mesh, lockedColors });
          }}
        />
      )}

      {/* A preset is its picture; it needs no name to be found again. */}
      <div className="flex justify-end">
        <Button
          isDisabled={isDisabled}
          onPress={() => {
            onSavePreset(value);
          }}
        >
          Save Preset
        </Button>
      </div>
    </div>
  );
}
