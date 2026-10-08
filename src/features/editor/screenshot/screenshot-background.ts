// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { randomMeshBackground } from "../../../components/shared/background-picker/background-random";

/** A fresh mesh in the flat fields the output settings carry it in. */
export const randomMeshComposition = (colorCount?: number) => {
  const mesh = randomMeshBackground(colorCount);
  return {
    meshColors: mesh.colors,
    meshPoints: mesh.points ?? [],
    meshSeed: mesh.seed,
    meshWarpPercent: mesh.warpPercent,
  };
};
