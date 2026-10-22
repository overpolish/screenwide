// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";

import { type Element } from "./ideas-data";
import { H, px, spring, W } from "./point-geometry";

// What the tools leave on the picture. A counter is a callout: it lands on
// its target first, with a ripple, then draws its leader out to a disc that
// springs open. An arrow lifts off the picture with a two-layer shadow.

const RED = "#ff453a";

export function Callout({
  element,
  number,
}: {
  element: Element;
  number: number;
}) {
  const rect = px(element);
  // The callout leans toward the middle of the picture, where there is
  // room: it lands on the element's corner facing that way, inside the
  // corner's curve so the dot sits on the element, not past it.
  const lean = {
    x: rect.x + rect.w / 2 < W / 2 ? 1 : -1,
    y: rect.y + rect.h / 2 < H / 2 ? 1 : -1,
  };
  const inset = element.radius * 0.45 + 6;
  const dot = {
    x: lean.x > 0 ? rect.x + rect.w - inset : rect.x + inset,
    y: lean.y > 0 ? rect.y + rect.h - inset : rect.y + inset,
  };
  const disc = { x: dot.x + 30 * lean.x, y: dot.y + 30 * lean.y };
  return (
    <g>
      <motion.circle
        animate={{ opacity: [0, 0.5, 0], r: [0, 9, 26] }}
        cx={dot.x}
        cy={dot.y}
        fill="none"
        stroke={RED}
        strokeWidth={1.5}
        transition={{ duration: 0.6 }}
      />
      <motion.circle
        animate={{ r: 5.5 }}
        cx={dot.x}
        cy={dot.y}
        fill="#fff"
        initial={{ r: 0 }}
        transition={spring}
      />
      <motion.circle
        animate={{ r: 3.5 }}
        cx={dot.x}
        cy={dot.y}
        fill={RED}
        initial={{ r: 0 }}
        transition={spring}
      />
      <motion.line
        animate={{ pathLength: 1 }}
        initial={{ pathLength: 0 }}
        stroke={RED}
        strokeWidth={1.6}
        transition={{ delay: 0.12, duration: 0.22, ease: "easeOut" }}
        x1={dot.x}
        x2={disc.x - 9 * lean.x}
        y1={dot.y}
        y2={disc.y - 9 * lean.y}
      />
      <motion.g
        animate={{ scale: 1 }}
        initial={{ scale: 0 }}
        style={{ transformBox: "fill-box", transformOrigin: "center" }}
        transition={{ ...spring, delay: 0.28 }}
      >
        <circle cx={disc.x} cy={disc.y} fill={RED} filter="url(#lift)" r={13} />
        <text
          dominantBaseline="central"
          fill="#fff"
          fontSize={14}
          fontWeight={700}
          textAnchor="middle"
          x={disc.x}
          y={disc.y + 0.5}
        >
          {number}
        </text>
      </motion.g>
    </g>
  );
}

type Point = { x: number; y: number };

export function Arrow({ from, to }: { from: Point; to: Point }) {
  const angle = Math.atan2(to.y - from.y, to.x - from.x);
  const bend = {
    x: (from.x + to.x) / 2 - Math.sin(angle) * 30,
    y: (from.y + to.y) / 2 + Math.cos(angle) * 30,
  };
  const back = (offset: number, spread: number) => ({
    x: to.x - Math.cos(angle) * offset + Math.cos(angle + Math.PI / 2) * spread,
    y: to.y - Math.sin(angle) * offset + Math.sin(angle + Math.PI / 2) * spread,
  });
  const [one, other, neck] = [back(16, 8), back(16, -8), back(12, 0)];
  return (
    <g filter="url(#lift)">
      <path
        d={`M${String(from.x)} ${String(from.y)} Q${String(bend.x)} ${String(bend.y)} ${String(neck.x)} ${String(neck.y)}`}
        fill="none"
        stroke={RED}
        strokeLinecap="round"
        strokeWidth={4}
      />
      <path
        d={`M${String(to.x)} ${String(to.y)} L${String(one.x)} ${String(one.y)} L${String(other.x)} ${String(other.y)} Z`}
        fill={RED}
        stroke={RED}
        strokeLinejoin="round"
        strokeWidth={2}
      />
    </g>
  );
}
