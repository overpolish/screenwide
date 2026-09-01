// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  fullHeight,
  glideGridRows,
  type GlideRegion,
  stepColumns,
  stepRows,
} from "./glide-regions";
import { axisStep } from "./glide-travel";

/** A window action the gesture arms and the lift commits, instead of a move. */
export type GlideAction = "minimize";

export type GlideFoldOptions = {
  /**
   * How square a flick has to be to read as aimed at a corner: the shorter
   * axis over the longer one, so 0.5 is a cone of roughly 27° to 63°.
   */
  diagonalCornerRatio: number;
  /** Ratio one axis must beat the other by to own the first fold. */
  horizontalDominance: number;
  /** Distance of the first horizontal fold, and of one horizontal step. */
  horizontalThreshold: number;
  /** Upward distance that folds straight into fill. */
  verticalFillThreshold: number;
  /** Cost of giving a single row back to full height: a deliberate release. */
  verticalReleaseThreshold: number;
  /** Cost of a vertical step from full height: a corner must be meant. */
  verticalThreshold: number;
};

export const defaultGlideFoldOptions: GlideFoldOptions = {
  diagonalCornerRatio: 0.5,
  horizontalDominance: 1.15,
  horizontalThreshold: 44,
  verticalFillThreshold: 44,
  verticalReleaseThreshold: 20,
  verticalThreshold: 44,
};

/** Everything a first fold is decided from: travel, policy, thresholds. */
export type GlideFoldInput = {
  /** Signed horizontal travel since the last turn point; positive is right. */
  across: number;
  /** Signed vertical travel since the last turn point; positive is down. */
  down: number;
  options: GlideFoldOptions;
  thirds: boolean;
};

/** What a first fold produced, and how its settle should behave. */
export type GlideFold = {
  pending: GlideAction | null;
  /** Whether the settle that follows stays open to the vertical axis. */
  porous: boolean;
  region: GlideRegion | null;
};

/** The sideways first fold: dominant travel takes the far column outright. */
export const foldHorizontal = ({
  across,
  down,
  options,
  thirds,
}: GlideFoldInput): GlideRegion | null => {
  const { horizontalDominance, horizontalThreshold } = options;
  const rival = Math.abs(down) * horizontalDominance;
  if (Math.abs(across) < Math.max(horizontalThreshold, rival)) return null;

  const gridCols = thirds ? 3 : 2;
  const colStart = across > 0 ? gridCols - 1 : 0;
  return { ...fullHeight, colSpan: 1, colStart, gridCols };
};

/**
 * The diagonal first fold: both axes past their own thresholds, and neither so
 * much longer than the other that the flick reads as a straight one. A flick
 * that means a corner lands on it in a single transition, rather than dying in
 * the dominance dead zone where no axis wins.
 */
const foldCorner = ({
  across,
  down,
  options,
  thirds,
}: GlideFoldInput): GlideRegion | null => {
  const { diagonalCornerRatio, horizontalThreshold, verticalThreshold } =
    options;
  const wide = Math.abs(across);
  const tall = Math.abs(down);
  if (wide < horizontalThreshold || tall < verticalThreshold) return null;
  if (Math.min(wide, tall) < Math.max(wide, tall) * diagonalCornerRatio) {
    return null;
  }

  const gridCols = thirds ? 3 : 2;
  return {
    colSpan: 1,
    colStart: across > 0 ? gridCols - 1 : 0,
    gridCols,
    rowSpan: 1,
    rowStart: down > 0 ? 1 : 0,
  };
};

/** The vertical first fold: up fills the screen, down arms the minimize. */
const foldVertical = ({
  across,
  down,
  options,
  thirds,
}: GlideFoldInput): GlideFold | null => {
  const { horizontalDominance, verticalFillThreshold } = options;
  const rival = Math.abs(across) * horizontalDominance;
  if (Math.abs(down) < Math.max(verticalFillThreshold, rival)) return null;

  // Straight down is fill's opposite: it arms a minimize that the lift
  // commits, in either grid. Folding on down again converts it to a row.
  if (down > 0) return { pending: "minimize", porous: false, region: null };

  // Straight up fills the screen; thirds takes the middle cell instead,
  // which the ladder also reaches in two steps.
  const gridCols = thirds ? 3 : 2;
  const middle = thirds ? 1 : 0;
  const colSpan = middle === 1 ? 1 : 2;
  return {
    pending: null,
    porous: false,
    region: { ...fullHeight, colSpan, colStart: middle, gridCols },
  };
};

/**
 * The opening transition, decided cone first: a genuinely diagonal flick is a
 * corner, so a down-right diagonal inside the cone is the bottom-right cell
 * rather than an armed minimize, and an up-left one is the top-left cell
 * rather than fill. Outside the cone the dominant-axis rules below decide,
 * unchanged - a straight-dominant down still arms, straight up still fills.
 */
export const detectFirstFold = (input: GlideFoldInput): GlideFold | null => {
  const corner = foldCorner(input);
  if (corner) return { pending: null, porous: false, region: corner };

  // Only the opening sideways fold leaves its settle porous, so an L-shaped
  // motion that turns without stopping can still buy its row: see
  // GlideDetector's porous settle.
  const sideways = foldHorizontal(input);
  if (sideways) return { pending: null, porous: true, region: sideways };

  return foldVertical(input);
};

/** The one step a ready window can buy over a region, or null when short. */
export const stepLadder = (
  from: GlideRegion,
  { across, down, options }: GlideFoldInput,
): GlideFold | null => {
  const { horizontalThreshold, verticalReleaseThreshold, verticalThreshold } =
    options;
  const sideways = axisStep(across, horizontalThreshold);
  if (sideways !== 0) {
    return {
      pending: null,
      porous: false,
      region: stepColumns(from, sideways),
    };
  }

  // Leaving a single row is cheap, reproducing the corner's easy release.
  const full = from.rowSpan === glideGridRows;
  const step = axisStep(
    down,
    full ? verticalThreshold : verticalReleaseThreshold,
  );
  if (step === 0) return null;

  // A down step that the grid cannot honour is at the bottom edge, where the
  // ladder loops: it arms the minimize over the region instead of no-opping.
  const region = stepRows(from, step);
  const looped = step > 0 && region === from;
  return { pending: looped ? "minimize" : null, porous: false, region };
};
