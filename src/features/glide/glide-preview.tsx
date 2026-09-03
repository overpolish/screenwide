// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, useAnimate } from "motion/react";
import { useEffect } from "react";

import { type GlideAction } from "./glide-detection";
import {
  describeRegion,
  glideGridRows,
  type GlideRegion,
} from "./glide-regions";

const percent = (value: number, of: number) =>
  `${((value / of) * 100).toFixed(4)}%`;

const interiorCornerClasses = (region: GlideRegion) => {
  const reachesBottom = region.rowStart + region.rowSpan === glideGridRows;
  const reachesRight = region.colStart + region.colSpan === region.gridCols;

  return [
    region.rowStart > 0 && region.colStart > 0 ? "rounded-tl-sm" : null,
    region.rowStart > 0 && !reachesRight ? "rounded-tr-sm" : null,
    !reachesBottom && region.colStart > 0 ? "rounded-bl-sm" : null,
    !reachesBottom && !reachesRight ? "rounded-br-sm" : null,
  ]
    .filter(Boolean)
    .join(" ");
};

const destinationGeometry = (
  region: GlideRegion | null,
  pending: GlideAction | null,
) => {
  if (pending === "minimize") {
    return {
      corners: "rounded-t-sm",
      height: "12.5%",
      left: "33.3333%",
      top: "87.5%",
      width: "33.3333%",
    };
  }
  if (!region) return null;
  return {
    corners: interiorCornerClasses(region),
    height: percent(region.rowSpan, glideGridRows),
    left: percent(region.colStart, region.gridCols),
    top: percent(region.rowStart, glideGridRows),
    width: percent(region.colSpan, region.gridCols),
  };
};

/** What the last settle achieved, in work-area fractions. */
export type GlideFit = {
  actual: { height: number; width: number; x: number; y: number };
  fits: boolean;
};

type GlideRect = GlideFit["actual"];

/** Places a normalized work-area rectangle in the preview's percentage space. */
const rectGeometry = (rect: GlideRect) => ({
  height: `${(rect.height * 100).toFixed(4)}%`,
  left: `${(rect.x * 100).toFixed(4)}%`,
  top: `${(rect.y * 100).toFixed(4)}%`,
  width: `${(rect.width * 100).toFixed(4)}%`,
});

const regionRect = (region: GlideRegion): GlideRect => ({
  height: region.rowSpan / glideGridRows,
  width: region.colSpan / region.gridCols,
  x: region.colStart / region.gridCols,
  y: region.rowStart / glideGridRows,
});

/** The part of the actual frame that satisfied the requested destination. */
const intersect = (first: GlideRect, second: GlideRect): GlideRect | null => {
  const x = Math.max(first.x, second.x);
  const y = Math.max(first.y, second.y);
  const right = Math.min(first.x + first.width, second.x + second.width);
  const bottom = Math.min(first.y + first.height, second.y + second.height);
  if (right <= x || bottom <= y) return null;
  return { height: bottom - y, width: right - x, x, y };
};

/** Only corners that do not meet the work-area edge receive a radius. */
const rectCornerClasses = (rect: GlideRect) => {
  const reachesBottom = rect.y + rect.height >= 0.9999;
  const reachesRight = rect.x + rect.width >= 0.9999;

  return [
    rect.y > 0.0001 && rect.x > 0.0001 ? "rounded-tl-sm" : null,
    rect.y > 0.0001 && !reachesRight ? "rounded-tr-sm" : null,
    !reachesBottom && rect.x > 0.0001 ? "rounded-bl-sm" : null,
    !reachesBottom && !reachesRight ? "rounded-br-sm" : null,
  ]
    .filter(Boolean)
    .join(" ");
};

/** Names what the preview is showing, for the aria label. */
const describeDestination = (
  region: GlideRegion | null,
  pending: GlideAction | null,
) => {
  // An armed minimize wins: the region underneath is what an up step returns
  // to, not where the lift would place the window.
  if (pending === "minimize") return "Minimize";
  return region
    ? `Glide destination: ${describeRegion(region)}`
    : "No Glide destination";
};

export function GlidePreview({
  fit,
  iconSrc,
  pending,
  pulse,
  region,
}: {
  /** The last settle's report, or null while a destination is in flight. */
  fit: GlideFit | null;
  /** The glided app's icon, once it resolves; decorative, so no alt text. */
  iconSrc: null | string;
  pending: GlideAction | null;
  /** Counts the rests completed; each one plays the ready breath. */
  pulse: number;
  region: GlideRegion | null;
}) {
  const [segment, animate] = useAnimate();
  const destination = destinationGeometry(region, pending);
  // An app that could not fill its region gets the truth: a quiet destination
  // surface behind the primary surface showing the extent it actually reached.
  // A pending arm is showing its own hint, so it keeps the primary fill.
  const constrained =
    fit && !fit.fits && region && pending === null ? fit : null;
  const overlap =
    constrained && region
      ? intersect(regionRect(region), constrained.actual)
      : null;

  // The visual sibling of the haptic tick, for hands that cannot feel it:
  // a brief semantic state flash on the destination itself.
  useEffect(() => {
    if (pulse === 0 || !segment.current) return;
    void animate(
      segment.current,
      {
        backgroundColor: [
          "var(--color-primary-surface)",
          "var(--color-primary-surface-hover)",
          "var(--color-primary-surface)",
        ],
      },
      { duration: 0.12, ease: "easeOut", times: [0, 0.4, 1] },
    );
  }, [animate, pulse, segment]);

  return (
    <div
      aria-label={describeDestination(region, pending)}
      className="window-surface rounded-window relative h-full w-full overflow-hidden"
      role="img"
    >
      {destination ? (
        <motion.div
          animate={{
            backgroundColor: constrained
              ? "var(--color-neutral)"
              : "var(--color-primary-surface)",
            height: destination.height,
            left: destination.left,
            top: destination.top,
            width: destination.width,
          }}
          className={`absolute bg-primary-surface ${destination.corners}`}
          initial={false}
          ref={constrained ? undefined : segment}
          transition={{ duration: 0.06, ease: "easeOut" }}
        />
      ) : null}
      {/* The requested and actual frames are both neutral; only their overlap
          is primary, so every kind of mismatch reads consistently. */}
      {constrained ? (
        <>
          <motion.div
            animate={{ opacity: 1 }}
            className={`absolute bg-neutral ${rectCornerClasses(constrained.actual)}`}
            initial={{ opacity: 0 }}
            style={rectGeometry(constrained.actual)}
            transition={{ duration: 0.06, ease: "easeOut" }}
          />
          {overlap ? (
            <motion.div
              animate={{ opacity: 1 }}
              className={`absolute bg-primary-surface ${rectCornerClasses(overlap)}`}
              initial={{ opacity: 0 }}
              ref={segment}
              style={rectGeometry(overlap)}
              transition={{ duration: 0.06, ease: "easeOut" }}
            />
          ) : null}
        </>
      ) : null}
      {/* Centered over whatever the fill is doing: which app is moving is one
          fact about the whole preview, not about the destination. */}
      {iconSrc ? (
        <>
          {/* Some Windows executables expose unusually low-alpha icon artwork.
              A second identical layer restores its visual weight without
              changing opaque icons or baking in an app-specific backdrop. */}
          <img
            alt=""
            className="glide-app-icon-windows-boost pointer-events-none absolute inset-0 m-auto size-icon-default object-contain"
            src={iconSrc}
          />
          <img
            alt=""
            className="pointer-events-none absolute inset-0 m-auto size-icon-default object-contain"
            src={iconSrc}
          />
        </>
      ) : null}
    </div>
  );
}
