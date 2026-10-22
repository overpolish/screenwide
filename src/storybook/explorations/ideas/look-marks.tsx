// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";

import { draw, handLoop, type Look, pop } from "./looks";
import { H, W } from "./point-geometry";

// How each look draws a mark. Product is ink and hairlines, lifted by a soft
// two-layer shadow; Editorial is pen on paper, rough-edged and multiplied in;
// Playful is bold and rounded with a sticker's offset shadow. Each draws on
// the way a hand would, so a new mark arrives rather than appears.

type Point = { x: number; y: number };
export type Common = { color: string; look: Look; scale: number };

/** Defs the marks refer to: the lift shadow and the marker's rough edge. */
export function MarkDefs() {
  return (
    <defs>
      <filter height="200%" id="look-lift" width="200%" x="-50%" y="-50%">
        <feDropShadow dx={0} dy={1} floodOpacity={0.28} stdDeviation={0.8} />
        <feDropShadow dx={0} dy={4} floodOpacity={0.16} stdDeviation={5} />
      </filter>
      <filter id="look-rough">
        <feTurbulence
          baseFrequency={0.9}
          numOctaves={2}
          seed={4}
          type="fractalNoise"
        />
        <feDisplacementMap in="SourceGraphic" scale={2.4} />
      </filter>
    </defs>
  );
}

export function ArrowMark({
  color,
  from,
  look,
  scale,
  to,
}: Common & { from: Point; to: Point }) {
  const angle = Math.atan2(to.y - from.y, to.x - from.x);
  const bend = {
    x: (from.x + to.x) / 2 - Math.sin(angle) * 26,
    y: (from.y + to.y) / 2 + Math.cos(angle) * 26,
  };
  const head = (back: number, spread: number) => {
    const x =
      to.x - Math.cos(angle) * back + Math.cos(angle + Math.PI / 2) * spread;
    const y =
      to.y - Math.sin(angle) * back + Math.sin(angle + Math.PI / 2) * spread;
    return `${x.toFixed(1)} ${y.toFixed(1)}`;
  };
  const path = `M${String(from.x)} ${String(from.y)} Q${String(bend.x)} ${String(bend.y)} ${String(to.x)} ${String(to.y)}`;
  if (look.id === "editorial") {
    return (
      <g filter="url(#look-rough)" style={{ mixBlendMode: "multiply" }}>
        <motion.path
          animate={{ pathLength: 1 }}
          d={path}
          fill="none"
          initial={{ pathLength: 0 }}
          stroke={color}
          strokeLinecap="round"
          strokeWidth={3.2 * scale}
          transition={draw}
        />
        <motion.path
          animate={{ opacity: 1 }}
          d={`M${head(15 * scale, 10 * scale)} L${String(to.x)} ${String(to.y)} L${head(13 * scale, -11 * scale)}`}
          fill="none"
          initial={{ opacity: 0 }}
          stroke={color}
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={3.2 * scale}
          transition={{ delay: 0.4, duration: 0.1 }}
        />
      </g>
    );
  }
  if (look.id === "playful") {
    return (
      <g>
        <motion.path
          animate={{ pathLength: 1 }}
          d={path}
          fill="none"
          initial={{ pathLength: 0 }}
          stroke={color}
          strokeLinecap="round"
          strokeWidth={6 * scale}
          transition={draw}
        />
        <motion.path
          animate={{ scale: 1 }}
          d={`M${String(to.x)} ${String(to.y)} L${head(17 * scale, 11 * scale)} L${head(17 * scale, -11 * scale)} Z`}
          fill={color}
          initial={{ scale: 0 }}
          stroke={color}
          strokeLinejoin="round"
          strokeWidth={4 * scale}
          style={{ transformBox: "fill-box", transformOrigin: "center" }}
          transition={{ ...pop, delay: 0.35 }}
        />
      </g>
    );
  }
  return (
    <g filter="url(#look-lift)">
      <motion.path
        animate={{ pathLength: 1 }}
        d={path}
        fill="none"
        initial={{ pathLength: 0 }}
        stroke="#fff"
        strokeLinecap="round"
        strokeWidth={5.5 * scale}
        transition={draw}
      />
      <motion.path
        animate={{ pathLength: 1 }}
        d={path}
        fill="none"
        initial={{ pathLength: 0 }}
        stroke={color}
        strokeLinecap="round"
        strokeWidth={2 * scale}
        transition={draw}
      />
      <path
        d={`M${head(10 * scale, 6.5 * scale)} L${String(to.x)} ${String(to.y)} L${head(10 * scale, -6.5 * scale)}`}
        fill="none"
        stroke={color}
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2 * scale}
      />
    </g>
  );
}

export function FocusMark({
  color,
  look,
  radius,
  rect,
  scale,
}: Common & {
  radius: number;
  rect: { h: number; w: number; x: number; y: number };
}) {
  const pad = 6 * scale;
  if (look.id === "editorial") {
    const centre = { x: rect.x + rect.w / 2, y: rect.y + rect.h / 2 };
    return (
      <motion.path
        animate={{ pathLength: 1 }}
        d={handLoop({
          centre,
          rx: rect.w / 2 + 14,
          ry: rect.h / 2 + 10,
          seed: 2,
        })}
        fill="none"
        filter="url(#look-rough)"
        initial={{ pathLength: 0 }}
        stroke={color}
        strokeLinecap="round"
        strokeWidth={3 * scale}
        style={{ mixBlendMode: "multiply" }}
        transition={{ ...draw, duration: 0.7 }}
      />
    );
  }
  if (look.id === "playful") {
    return (
      <motion.rect
        animate={{ opacity: 1, scale: 1 }}
        fill="none"
        height={rect.h + pad * 2}
        initial={{ opacity: 0, scale: 1.15 }}
        rx={radius + pad}
        stroke={color}
        strokeWidth={4 * scale}
        style={{ transformBox: "fill-box", transformOrigin: "center" }}
        transition={pop}
        width={rect.w + pad * 2}
        x={rect.x - pad}
        y={rect.y - pad}
      />
    );
  }
  // Product focuses by taking the rest away, not by drawing round it.
  return (
    <g>
      <mask id={`focus-${String(rect.x)}-${String(rect.y)}`}>
        <rect fill="#fff" height={H} width={W} />
        <rect
          fill="#000"
          height={rect.h + pad * 2}
          rx={radius + pad}
          width={rect.w + pad * 2}
          x={rect.x - pad}
          y={rect.y - pad}
        />
      </mask>
      <motion.rect
        animate={{ opacity: 1 }}
        fill="rgba(0,0,0,0.42)"
        height={H}
        initial={{ opacity: 0 }}
        mask={`url(#focus-${String(rect.x)}-${String(rect.y)})`}
        transition={{ duration: 0.3 }}
        width={W}
      />
    </g>
  );
}
