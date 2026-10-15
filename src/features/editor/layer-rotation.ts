// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { SceneRotation } from "../../bindings/SceneRotation";

export type { SceneRotation };

/** Standing upright: how a layer, or a scene, turns it until it is turned. */
export const UPRIGHT_ROTATION: SceneRotation = { x: 0, y: 0, z: 0 };

/** The furthest a layer or a scene turns about any axis, in degrees: ten
 * whole turns either way. The twin of `MAX_DEGREES` in `rotation.rs`. */
export const MAX_TURN = 3600;

/** `value` turned by `rotation`, each angle held to what a turn can reach;
 * one that leaves it upright keeps no turn at all. */
export const withRotation = <Turned extends { rotation?: SceneRotation }>(
  value: Turned,
  rotation: SceneRotation,
): Turned => {
  const { rotation: _rotation, ...rest } = value;
  const held = (angle: number) =>
    Math.min(MAX_TURN, Math.max(-MAX_TURN, angle));
  const turned = {
    x: held(rotation.x),
    y: held(rotation.y),
    z: held(rotation.z),
  };
  const upright = turned.x === 0 && turned.y === 0 && turned.z === 0;
  return (upright ? rest : { ...rest, rotation: turned }) as Turned;
};
