// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";
import { type ReactNode, useEffect, useRef, useState } from "react";

import { cn } from "../../../lib/styling";
import { Text } from "../../../primitives/text/text";

import { MockApp } from "./mock-app";
import { RecordedCursor } from "./recorded-cursor";

// Settings shown as what they do. Each card's preview is the setting: it is
// changed by touching the preview, and the line under it says the value in
// words.

function LivingCard({
  children,
  title,
  value,
}: {
  children: ReactNode;
  title: string;
  value: string;
}) {
  return (
    <div className="flex flex-col gap-inline rounded-panel bg-fill p-inset-sm">
      <div className="relative flex h-32 items-center justify-center overflow-hidden rounded-control bg-recess">
        {children}
      </div>
      <div className="flex items-baseline justify-between gap-inline px-1 pb-0.5">
        <Text variant="label">{title}</Text>
        <Text variant="caption">{value}</Text>
      </div>
    </div>
  );
}

/** Seconds since mount, ticking every frame. */
function useClock() {
  const [time, setTime] = useState(0);
  useEffect(() => {
    let frame = 0;
    const start = performance.now();
    const tick = (now: number) => {
      setTime((now - start) / 1000);
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
    };
  }, []);
  return time;
}

const COUNTDOWNS = [0, 3, 5];

export function CountdownCard() {
  const [seconds, setSeconds] = useState(3);
  const time = useClock();
  const cycle = seconds + 1.2;
  const left = seconds - Math.floor(time % cycle);
  return (
    <LivingCard
      title="Countdown"
      value={seconds === 0 ? "Records at once" : `${String(seconds)} seconds`}
    >
      <button
        aria-label="Countdown"
        className="flex size-20 items-center justify-center rounded-full bg-[#1c1c1e] text-white shadow-lg"
        onClick={() => {
          setSeconds(
            COUNTDOWNS[(COUNTDOWNS.indexOf(seconds) + 1) % COUNTDOWNS.length],
          );
        }}
        type="button"
      >
        {seconds === 0 || left <= 0 ? (
          <span className="size-4 rounded-full bg-error" />
        ) : (
          <motion.span
            animate={{ opacity: 1, scale: 1 }}
            className="text-[30px] font-semibold tabular-nums"
            initial={{ opacity: 0, scale: 1.4 }}
            key={left}
          >
            {left}
          </motion.span>
        )}
      </button>
    </LivingCard>
  );
}

export function FrameRateCard() {
  const [fps, setFps] = useState(60);
  const time = useClock();
  return (
    <LivingCard title="Frame rate" value={`${String(fps)} frames a second`}>
      <div className="flex w-full flex-col gap-2 px-inset" dir="ltr">
        {[30, 60].map((rate) => {
          const stepped = Math.floor(time * rate) / rate;
          const x = (Math.sin(stepped * 2.6) * 0.5 + 0.5) * 100;
          return (
            <button
              aria-pressed={fps === rate}
              className={cn(
                "relative h-10 rounded-control ring-2 transition-shadow",
                fps === rate
                  ? "bg-fill-strong ring-primary"
                  : "ring-transparent hover:bg-fill",
              )}
              key={rate}
              onClick={() => {
                setFps(rate);
              }}
              type="button"
            >
              <span className="absolute inset-y-0 start-2 flex items-center text-caption text-fg-tertiary">
                {rate}
              </span>
              <span
                className="absolute top-1/2 size-4 -translate-y-1/2 rounded-full bg-fg"
                style={{
                  insetInlineStart: `calc(28px + (100% - 48px) * ${String(x / 100)})`,
                }}
              />
            </button>
          );
        })}
      </div>
    </LivingCard>
  );
}

export function FileSizeCard() {
  const [isShown, setIsShown] = useState(false);
  return (
    <LivingCard
      title="File size while recording"
      value={isShown ? "Shown" : "Hidden"}
    >
      <motion.div
        className="flex items-center gap-2 rounded-full bg-[#1c1c1e] py-2 ps-3 pe-2 text-label text-white shadow-lg"
        layout
        transition={{ bounce: 0.2, duration: 0.4, type: "spring" }}
      >
        <span className="size-2 rounded-full bg-error" />
        <span className="tabular-nums">00:42</span>
        <motion.button
          aria-pressed={isShown}
          className={cn(
            "rounded-full px-2 py-0.5 tabular-nums transition-colors",
            isShown
              ? "text-white/60 hover:bg-white/10"
              : "border border-dashed border-white/30 text-white/30 hover:text-white/60",
          )}
          layout
          onClick={() => {
            setIsShown((value) => !value);
          }}
          type="button"
        >
          48 MB
        </motion.button>
      </motion.div>
      <span className="absolute bottom-2 text-caption text-fg-tertiary">
        Click the size to {isShown ? "hide" : "show"} it
      </span>
    </LivingCard>
  );
}

export function AutoZoomCard() {
  const [isOn, setIsOn] = useState(true);
  const time = useClock();
  // The pointer clicks Create every four seconds; with zoom on, the view
  // follows it in and back out.
  const phase = time % 4;
  const zoom = isOn
    ? 1 +
      0.9 *
        Math.max(
          0,
          Math.sin(Math.min(Math.max((phase - 1.2) / 2.2, 0), 1) * Math.PI),
        )
    : 1;
  const focus = { x: 83.5, y: 65.5 };
  return (
    <LivingCard
      title="Zoom in automatically"
      value={isOn ? "Follows your clicks" : "Off"}
    >
      <button
        aria-pressed={isOn}
        className="h-[120px] w-[192px] overflow-hidden rounded-[8px] shadow-md"
        onClick={() => {
          setIsOn((value) => !value);
        }}
        type="button"
      >
        <div className="w-[480px] origin-top-left scale-[0.4]">
          <MockApp
            style={{
              transform: `scale(${String(zoom)})`,
              transformOrigin: `${String(focus.x)}% ${String(focus.y)}%`,
            }}
          >
            <RecordedCursor seconds={24.9 + Math.min(phase, 1.25)} />
          </MockApp>
        </div>
      </button>
    </LivingCard>
  );
}

export function WindowGapCard() {
  const [gap, setGap] = useState(12);
  const dragRef = useRef<{ from: number; start: number } | null>(null);
  return (
    <LivingCard title="Space around windows" value={`${String(gap)} px`}>
      <div
        className="absolute inset-3 rounded-[8px] bg-[linear-gradient(135deg,#7aa7ff,#c38bff)]"
        dir="ltr"
      >
        <div className="absolute inset-0 flex" style={{ gap, padding: gap }}>
          <span className="grow rounded-[6px] bg-white/90 shadow" />
          <span className="grow rounded-[6px] bg-white/90 shadow" />
        </div>
        <span
          aria-label="Space around windows"
          aria-valuemax={32}
          aria-valuemin={0}
          aria-valuenow={gap}
          className="absolute inset-y-0 start-1/2 w-6 -translate-x-1/2 cursor-ew-resize touch-none"
          onPointerDown={(event) => {
            event.currentTarget.setPointerCapture(event.pointerId);
            dragRef.current = { from: event.clientX, start: gap };
          }}
          onPointerMove={(event) => {
            const current = dragRef.current;
            if (!current) return;
            setGap(
              Math.round(
                Math.min(
                  Math.max(current.start + (event.clientX - current.from), 0),
                  32,
                ),
              ),
            );
          }}
          onPointerUp={() => {
            dragRef.current = null;
          }}
          role="slider"
        >
          <span className="absolute inset-y-6 start-1/2 w-1 -translate-x-1/2 rounded-full bg-white/80" />
        </span>
      </div>
    </LivingCard>
  );
}
