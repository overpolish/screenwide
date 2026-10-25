// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { progress } from "./loop-timing";
import { ToolScreen } from "./tool-screen";
import { useClock } from "./use-clock";

// The Ruler, Annotate and OCR pictures in Tools, each the tool at work on a
// loop.

/** Two cards measured: the distance between them and the size of one, as
 * the ruler labels them, with the crosshair drifting over the screen. */
export function RulerArt({ isOn }: { isOn: boolean }) {
  const time = useClock();
  const drift = 40 + Math.sin(time * 0.8) * 12;
  return (
    <ToolScreen isOn={isOn}>
      <span className="absolute start-[18%] top-[22%] h-[56%] w-[24%] rounded-[5px] bg-white/95 shadow-md" />
      <span className="absolute start-[58%] top-[30%] h-[40%] w-[22%] rounded-[5px] bg-white/95 shadow-md" />
      {isOn ? (
        <>
          <span
            className="absolute inset-y-0 w-px bg-emphasis/70"
            style={{ insetInlineStart: `${String(drift)}%` }}
          />
          <span className="absolute inset-x-0 top-1/2 h-px bg-emphasis/70" />
          <span className="absolute start-[42%] top-1/2 h-px w-[16%] bg-emphasis" />
          <Label x="50%" y="calc(50% - 22px)">
            128 px
          </Label>
          <span className="absolute start-[83%] top-[30%] h-[40%] w-px bg-emphasis" />
          <Label x="calc(83% + 28px)" y="calc(50% - 9px)">
            96
          </Label>
        </>
      ) : null}
    </ToolScreen>
  );
}

function Label({
  children,
  x,
  y,
}: {
  children: ReactNode;
  x: string;
  y: string;
}) {
  return (
    <span
      className="absolute -translate-x-1/2 rounded-[4px] bg-emphasis px-1 py-px text-[10px] leading-[14px] font-semibold text-white tabular-nums"
      style={{ insetInlineStart: x, top: y }}
    >
      {children}
    </span>
  );
}

/** An arrow drawn to a button and a counter beside it, in the colour new
 * annotations take. Kept, they stay put through the loop; otherwise they
 * go when drawing ends. */
export function AnnotateArt({
  color,
  isKept,
  isOn,
}: {
  color: string;
  isKept: boolean;
  isOn: boolean;
}) {
  const time = useClock();
  const at = time % 4.4;
  const drawn = progress(at, 0.3, 1.3);
  const gone = isKept ? 0 : progress(at, 3.2, 3.6);
  // Drawn at a fixed size in the middle of the screen, so the arrow keeps
  // its shape however wide the pane is.
  return (
    <ToolScreen isOn={isOn}>
      <svg
        className="absolute start-1/2 top-1/2 -translate-1/2"
        height={96}
        viewBox="0 0 320 96"
        width={320}
      >
        <rect fill="#fff" height={76} rx={5} width={150} x={90} y={10} />
        <rect fill="#1d1d1f" height={16} rx={4} width={48} x={180} y={60} />
        {isOn ? (
          <g opacity={1 - gone}>
            <path
              d="M 28 14 Q 52 86 168 70"
              fill="none"
              pathLength={1}
              stroke={color}
              strokeDasharray="1"
              strokeDashoffset={1 - drawn}
              strokeLinecap="round"
              strokeWidth={4}
            />
            <path
              d="M 156 61 L 170 70 L 157 79"
              fill="none"
              opacity={drawn > 0.97 ? 1 : 0}
              stroke={color}
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={4}
            />
            <circle
              cx={268}
              cy={30}
              fill={color}
              opacity={progress(at, 1.4, 1.6)}
              r={11}
            />
            <text
              fill="#000"
              fontSize={12}
              fontWeight={700}
              opacity={progress(at, 1.4, 1.6)}
              textAnchor="middle"
              x={268}
              y={34}
            >
              1
            </text>
          </g>
        ) : null}
      </svg>
    </ToolScreen>
  );
}

const LINES = [
  "Launch checklist",
  "Record the demo at 60 fps",
  "Send captions for review",
  "Ship on Thursday",
];

/** A note and a QR code: the text is selected line by line as it is read,
 * and the code is framed as found. Drawn at a fixed size in the middle of
 * the screen, like the annotation. */
export function OcrArt({ isOn }: { isOn: boolean }) {
  const time = useClock();
  const at = time % 3.6;
  return (
    <ToolScreen isOn={isOn}>
      <div className="absolute start-1/2 top-1/2 h-24 w-80 -translate-1/2">
        <div className="absolute start-6 top-2.5 flex h-[76px] w-48 flex-col justify-center gap-0.5 rounded-[5px] bg-white px-2.5 text-[10px] leading-[14px] text-[#1d1d1f] shadow-md">
          {LINES.map((line, index) => (
            <span className="relative w-fit" key={line}>
              {isOn ? (
                <span
                  className="absolute inset-y-0 start-0 rounded-[2px] bg-[#b3d7ff]"
                  style={{
                    width: `${String(progress(at, 0.3 + index * 0.35, 0.65 + index * 0.35) * 100)}%`,
                  }}
                />
              ) : null}
              <span className="relative">{line}</span>
            </span>
          ))}
        </div>
        <svg
          className="absolute start-60 top-6 size-12 rounded-[4px] bg-white shadow-md"
          viewBox="0 0 12 12"
        >
          <path
            d="M2 2h2v2H2zM5 2h2v2H5zM8 5h2v2H8zM5 5h2v2H5zM2 8h2v2H2zM8 8h2v2H8zM5 8h2v2H5z"
            fill="#000"
          />
        </svg>
        {isOn ? (
          <span
            className="absolute start-[236px] top-5 size-14 rounded-[6px] border-2 border-emphasis"
            style={{ opacity: progress(at, 1.8, 2.1) }}
          />
        ) : null}
      </div>
    </ToolScreen>
  );
}
