// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";

import { type Common } from "./look-marks";
import { draw, handLoop, pop } from "./looks";

// A counter in each look: Product's ink badge and Playful's sticker sit on
// the element's corner; Editorial writes its numeral beside it and rings it
// by hand.

type Point = { x: number; y: number };

export function CounterMark({
  at,
  color,
  lean,
  look,
  number,
  scale,
}: Common & { at: Point; lean: Point; number: number }) {
  // Product and Playful sit on the corner, as a badge does; Editorial writes
  // its numeral beside the element and rings it.
  const reach = look.id === "editorial" ? 26 * scale : 0;
  const disc = { x: at.x + reach * lean.x, y: at.y + reach * lean.y };
  if (look.id === "editorial") {
    const r = 14 * scale;
    return (
      <g style={{ mixBlendMode: "multiply" }}>
        <motion.path
          animate={{ pathLength: 1 }}
          d={handLoop({ centre: disc, rx: r * 1.08, ry: r, seed: number })}
          fill="none"
          filter="url(#look-rough)"
          initial={{ pathLength: 0 }}
          stroke={color}
          strokeLinecap="round"
          strokeWidth={2.6 * scale}
          transition={draw}
        />
        <text
          dominantBaseline="central"
          fill={color}
          fontSize={17 * scale}
          fontWeight={700}
          textAnchor="middle"
          x={disc.x}
          y={disc.y + 1}
        >
          {number}
        </text>
      </g>
    );
  }
  const r = (look.id === "playful" ? 14 : 11) * scale;
  return (
    <motion.g
      animate={{ scale: 1 }}
      initial={{ scale: 0 }}
      style={{ transformBox: "fill-box", transformOrigin: "center" }}
      transition={pop}
    >
      {look.id === "playful" ? (
        <circle
          cx={disc.x}
          cy={disc.y + 3 * scale}
          fill="#000"
          opacity={0.18}
          r={r}
        />
      ) : (
        <circle
          cx={disc.x}
          cy={disc.y}
          fill="#fff"
          filter="url(#look-lift)"
          r={r + 1.5}
        />
      )}
      <circle cx={disc.x} cy={disc.y} fill={color} r={r} />
      <text
        dominantBaseline="central"
        fill="#fff"
        fontSize={(look.id === "playful" ? 15 : 12) * scale}
        fontWeight={look.id === "playful" ? 800 : 600}
        textAnchor="middle"
        x={disc.x}
        y={disc.y + 0.5}
      >
        {number}
      </text>
    </motion.g>
  );
}
