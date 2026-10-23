// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";
import { useMeter, useNumberFormatter } from "react-aria";

import { t } from "../../i18n/i18n";
import { cn } from "../../lib/styling";

import { decibelLevel, FLOOR_DECIBELS } from "./decibel-level";

const decibelToPercentage = (decibel: number): number =>
  decibelLevel(decibel) * 100;

let nextMeterId = 0;

const ticksForLength = (length: number) => {
  const ticks = [-48, -24];
  if (length > 70) ticks.push(-12);
  if (length > 95) ticks.push(-3);
  return ticks;
};

type TickProps = {
  tick: number;
  orientation?: "horizontal" | "vertical";
};
/**
 * Native level indicators draw unlabelled tick marks: a hairline per stop,
 * in the track's colour, so they read as the track's scale.
 * Vertical marks are anchored by `bottom`, so centring them shifts downward.
 */
const Tick = ({ orientation = "horizontal", tick }: TickProps) => {
  const percentage = decibelToPercentage(tick);
  const vertical = orientation === "vertical";
  return (
    <div
      className={`pointer-events-none absolute bg-fill select-none ${
        vertical
          ? "h-px w-1 translate-y-1/2"
          : "h-1 w-px -translate-x-1/2 rtl:translate-x-1/2"
      }`}
      style={
        vertical
          ? { bottom: `${percentage.toString()}%` }
          : { insetInlineStart: `${percentage.toString()}%` }
      }
    />
  );
};

type AudioMeterProps = {
  "aria-label": string;
  decibels: number;
  /** Thickness in pixels. */
  thickness: number;
  disabled?: boolean;
  hidePeakTick?: boolean;
  hideTicks?: boolean;
  /** Length in pixels, or a CSS length. */
  length?: number | string;
  orientation?: "horizontal" | "vertical";
  /** Peak in decibels. */
  peak?: number;
};

export const AudioMeter = ({
  "aria-label": label,
  decibels,
  disabled,
  hidePeakTick,
  hideTicks,
  length = "100%",
  orientation = "horizontal",
  peak = -Infinity,
  thickness,
}: AudioMeterProps) => {
  const vertical = orientation === "vertical";
  const idRef = useRef<number | null>(null);
  idRef.current ??= nextMeterId++;
  const id = idRef.current;
  const fillId = `meter-fill-${id.toString()}`;
  const meterClipId = `meter-clip-${id.toString()}`;
  const peakClipId = `peak-clip-${id.toString()}`;
  const percentage = disabled ? 0 : decibelToPercentage(decibels);
  const peakPercentage = decibelToPercentage(Math.min(peak, -0.5));

  const level = disabled
    ? FLOOR_DECIBELS
    : Math.min(0, Math.max(FLOOR_DECIBELS, decibels));
  const formatter = useNumberFormatter({ maximumFractionDigits: 0 });
  const { meterProps } = useMeter({
    "aria-label": label,
    maxValue: 0,
    minValue: FLOOR_DECIBELS,
    value: level,
    valueLabel:
      level <= FLOOR_DECIBELS
        ? t("controls-audio-meter-silent")
        : t("controls-audio-meter-value", {
            // `|| 0` keeps a level just under full scale from reading "-0".
            decibels: formatter.format(Math.round(level) || 0),
          }),
  });

  const svgRef = useRef<SVGSVGElement>(null);
  const [ticks, setTicks] = useState(() =>
    ticksForLength(typeof length === "number" ? length : 0),
  );

  useEffect(() => {
    const svg = svgRef.current;
    if (!svg) return;

    const resizeObserver = new ResizeObserver(([entry]) => {
      setTicks(
        ticksForLength(
          vertical ? entry.contentRect.height : entry.contentRect.width,
        ),
      );
    });
    resizeObserver.observe(svg);

    return () => {
      resizeObserver.disconnect();
    };
  }, [vertical]);

  // No viewBox: the rects are laid out in CSS pixels, so the corners stay
  // round at any length instead of stretching with the meter. The small-mark
  // radius, as a swatch has, held to half the thickness so a hairline meter
  // stays a capsule rather than overflowing its own corners.
  const corner = `min(var(--radius-sm), ${String(thickness / 2)}px)`;
  const meterRect = {
    height: "100%",
    style: { rx: corner, ry: corner },
    width: "100%",
  };

  return (
    <div
      {...meterProps}
      className={cn(
        "pointer-events-none select-none",
        vertical && "flex items-stretch",
        vertical && typeof length !== "number" && "h-full",
      )}
    >
      {/* Using SVG due to layering divs with border-radius and linear gradient
       * causing bleeding */}
      <svg
        aria-hidden="true"
        // Drawn filling to the right, so a horizontal meter turns to fill
        // from the start in a right-to-left language.
        className={vertical ? undefined : "rtl:-scale-x-100"}
        height={vertical ? length : thickness}
        ref={svgRef}
        width={vertical ? thickness : length}
      >
        <defs>
          <linearGradient
            id={fillId}
            x1="0%"
            x2={vertical ? "0%" : "100%"}
            y1={vertical ? "100%" : "0%"}
            y2="0%"
          >
            {/* The level reads as a traffic light, the convention every
                level meter shares: green while it is good, amber from
                about -6 dB, red near clipping. */}
            <stop offset="0%" stopColor="var(--color-success)" />
            <stop offset="78%" stopColor="var(--color-success)" />
            <stop offset="87%" stopColor="var(--color-warning)" />
            <stop offset="93%" stopColor="var(--color-warning)" />
            <stop offset="96%" stopColor="var(--color-error)" />
            <stop offset="100%" stopColor="var(--color-error)" />
          </linearGradient>

          <clipPath id={meterClipId}>
            {vertical ? (
              <rect
                height={`${percentage.toString()}%`}
                width="100%"
                y={`${(100 - percentage).toString()}%`}
              />
            ) : (
              <rect height="100%" width={`${percentage.toString()}%`} />
            )}
          </clipPath>

          <clipPath id={peakClipId}>
            {!disabled &&
              peak >= FLOOR_DECIBELS &&
              (vertical ? (
                <rect
                  height="2px"
                  transform="translate(0,-1)"
                  width="100%"
                  y={`${(100 - peakPercentage).toString()}%`}
                />
              ) : (
                <rect
                  height="100%"
                  transform="translate(-1.5,0)"
                  width="2px"
                  x={`${peakPercentage.toString()}%`}
                />
              ))}
          </clipPath>
        </defs>

        <rect className="fill-fill" {...meterRect} />
        <rect
          clipPath={`url(#${meterClipId})`}
          fill={`url(#${fillId})`}
          {...meterRect}
        />
        <rect
          clipPath={`url(#${peakClipId})`}
          fill={`url(#${fillId})`}
          {...meterRect}
        />
      </svg>

      {(!hideTicks || !hidePeakTick) && (
        <div
          aria-hidden="true"
          className={
            vertical
              ? `relative ${hidePeakTick ? "w-1" : "w-8"}`
              : `relative ${hidePeakTick ? "h-1" : "h-6"}`
          }
        >
          {!hideTicks &&
            [...ticks].map((tick) => (
              <Tick key={tick} orientation={orientation} tick={tick} />
            ))}

          {!hidePeakTick && !disabled && peak >= FLOOR_DECIBELS && (
            <span
              className={`pointer-events-none absolute tabular-nums select-none ${
                vertical
                  ? `translate-y-1/2 text-caption text-fg-secondary ${hideTicks ? "" : "ms-inline"}`
                  : `-translate-x-1/2 text-caption text-fg-secondary rtl:translate-x-1/2 ${hideTicks ? "" : "mt-inline"}`
              }`}
              style={
                vertical
                  ? { bottom: `${peakPercentage.toString()}%` }
                  : { insetInlineStart: `${peakPercentage.toString()}%` }
              }
            >
              {peak.toFixed(1)}
            </span>
          )}
        </div>
      )}
    </div>
  );
};
