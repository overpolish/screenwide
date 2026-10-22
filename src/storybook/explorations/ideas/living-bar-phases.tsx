// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Camera,
  Flag,
  Mic,
  Pause,
  Play,
  Square,
  Trash2,
  Volume2,
} from "lucide-react";
import { motion } from "motion/react";

import { cn } from "../../../lib/styling";
import { FakeScreen } from "../shared/picture";

import { mmss } from "./ideas-data";
import { BreathingDot, GlyphMeter, Tape } from "./living-bar-parts";

// What the bar holds in each phase. The bar itself only changes shape; these
// fade across inside it.

export type Inputs = { camera: boolean; mic: boolean; system: boolean };

const swap = { duration: 0.15 };
const fade = {
  animate: { opacity: 1 },
  exit: { opacity: 0 },
  initial: { opacity: 0 },
  transition: swap,
};

function IconAction({
  icon: Icon,
  label,
  onPress,
  tone = "plain",
}: {
  icon: typeof Play;
  label: string;
  onPress: () => void;
  tone?: "plain" | "stop";
}) {
  return (
    <button
      aria-label={label}
      className={cn(
        "flex size-9 items-center justify-center rounded-full",
        tone === "stop"
          ? "bg-white text-[#1c1c1e] hover:bg-white/90"
          : "text-white/80 hover:bg-white/10 hover:text-white",
      )}
      onClick={onPress}
      title={label}
      type="button"
    >
      <Icon
        className="size-4"
        fill={tone === "stop" ? "currentColor" : "none"}
      />
    </button>
  );
}

export function ReadyContent({
  inputs,
  isSilent,
  onRecord,
  onToggle,
}: {
  inputs: Inputs;
  isSilent: boolean;
  onRecord: () => void;
  onToggle: (input: keyof Inputs) => void;
}) {
  return (
    <motion.div
      {...fade}
      className="flex items-center gap-1 p-1.5"
      layout="position"
    >
      <FakeScreen className="ms-1 me-2 h-8 w-13 rounded-[6px]" />
      <GlyphMeter
        icon={Mic}
        isOn={inputs.mic}
        isSilent={isSilent}
        label="Microphone"
        onToggle={() => {
          onToggle("mic");
        }}
        seed={0}
      />
      <GlyphMeter
        icon={Camera}
        isOn={inputs.camera}
        label="Camera"
        onToggle={() => {
          onToggle("camera");
        }}
        seed={2}
      />
      <GlyphMeter
        icon={Volume2}
        isOn={inputs.system}
        label="Computer sound"
        onToggle={() => {
          onToggle("system");
        }}
        seed={4}
      />
      <button
        aria-label="Record"
        className="ms-2 flex size-10 items-center justify-center rounded-full bg-white/10 hover:bg-white/20"
        onClick={onRecord}
        type="button"
      >
        <span className="size-4 rounded-full bg-error ring-2 ring-white/80" />
      </button>
    </motion.div>
  );
}

export function CountdownContent({
  count,
  onCancel,
}: {
  count: number;
  onCancel: () => void;
}) {
  return (
    <motion.button
      {...fade}
      aria-label="Cancel the countdown"
      className="relative flex size-16 items-center justify-center"
      layout="position"
      onClick={onCancel}
      type="button"
    >
      <svg
        aria-hidden
        className="absolute inset-1.5 -rotate-90"
        viewBox="0 0 52 52"
      >
        <motion.circle
          animate={{ pathLength: 0 }}
          cx={26}
          cy={26}
          fill="none"
          initial={{ pathLength: 1 }}
          key={count}
          r={23}
          stroke="white"
          strokeLinecap="round"
          strokeWidth={2.5}
          transition={{ duration: 0.9, ease: "linear" }}
        />
      </svg>
      <motion.span
        animate={{ opacity: 1, scale: 1 }}
        className="text-[26px] font-semibold tabular-nums"
        initial={{ opacity: 0, scale: 1.4 }}
        key={`n${String(count)}`}
        transition={{ bounce: 0.3, duration: 0.35, type: "spring" }}
      >
        {count}
      </motion.span>
    </motion.button>
  );
}

export function LiveContent({
  isPaused,
  isSilent,
  moments,
  onDiscard,
  onMoment,
  onPause,
  onStop,
  seconds,
  showsControls,
}: {
  isPaused: boolean;
  isSilent: boolean;
  moments: number[];
  onDiscard: () => void;
  onMoment: () => void;
  onPause: () => void;
  onStop: () => void;
  seconds: number;
  showsControls: boolean;
}) {
  return (
    <motion.div
      {...fade}
      className="flex flex-col gap-2 p-1.5"
      layout="position"
    >
      {showsControls ? (
        <motion.div
          animate={{ opacity: 1 }}
          className="px-3 pt-2.5"
          initial={{ opacity: 0 }}
        >
          <Tape moments={moments} seconds={seconds} />
        </motion.div>
      ) : null}
      <div className="flex items-center gap-1">
        <span className="flex items-center gap-2 ps-2.5 pe-2">
          <BreathingDot isPaused={isPaused} isSilent={isSilent} />
          <span
            className={cn(
              "text-label tabular-nums",
              isPaused && "text-white/60",
            )}
          >
            {mmss(seconds)}
          </span>
        </span>
        {showsControls ? (
          <>
            <IconAction icon={Flag} label="Moment" onPress={onMoment} />
            <IconAction
              icon={isPaused ? Play : Pause}
              label={isPaused ? "Resume" : "Pause"}
              onPress={onPause}
            />
            <IconAction icon={Trash2} label="Discard" onPress={onDiscard} />
            <IconAction
              icon={Square}
              label="Stop"
              onPress={onStop}
              tone="stop"
            />
          </>
        ) : null}
      </div>
    </motion.div>
  );
}

export function DoneContent({
  moments,
  onAgain,
  seconds,
}: {
  moments: number;
  onAgain: () => void;
  seconds: number;
}) {
  return (
    <motion.div
      {...fade}
      className="flex w-80 flex-col gap-2.5 p-2.5"
      layout="position"
    >
      <FakeScreen className="aspect-video w-full rounded-[10px]" />
      <div className="flex items-center gap-2 px-1">
        <span className="grow text-label">
          {mmss(seconds)}
          <span className="text-white/50">
            {moments > 0 ? `, ${String(moments)} moments` : ""}
          </span>
        </span>
        <button
          className="rounded-full px-3 py-1.5 text-label whitespace-nowrap text-white/80 hover:bg-white/10"
          onClick={onAgain}
          type="button"
        >
          Record again
        </button>
        <button
          className="rounded-full bg-white px-3.5 py-1.5 text-label text-[#1c1c1e] hover:bg-white/90"
          type="button"
        >
          Edit
        </button>
      </div>
    </motion.div>
  );
}
