// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import "./combined.css";

import { ChevronDown, SlidersHorizontal } from "lucide-react";

import { cn } from "../../../../lib/styling";
import {
  DEFAULT_INPUTS,
  INPUTS,
  TARGETS,
} from "../../recorder/shared/recorder-data";
import { TargetStill } from "../../recorder/shared/recorder-parts";

import { Btn, IconBtn, Toggle } from "./combined-controls";

// The panel's content only: the window draws its own material and corners.
// The recorder as the Preflight Panel exploration settled it, in this look.

export function CombinedPreflight() {
  return (
    <div className="look-combined flex w-80 flex-col gap-3 p-3 font-sans text-[13px] text-(--fg)">
      <div className="grid grid-cols-4 gap-1">
        {TARGETS.map(({ icon: Icon, id, label }) => (
          <span
            className={cn(
              "flex flex-col items-center gap-1 rounded-lg py-1.5 text-[11px] text-(--fg-2)",
              id === "display" && "combined-raised text-(--fg)",
            )}
            key={id}
          >
            <Icon className="size-4" />
            {label}
          </span>
        ))}
      </div>
      <TargetStill
        className="aspect-video overflow-hidden rounded-xl"
        target="display"
      />
      <div className="grid grid-cols-3 gap-1.5">
        {INPUTS.map(({ device, icon: Icon, id, label, offIcon: OffIcon }) => {
          const isOn = DEFAULT_INPUTS[id];
          const Glyph = isOn ? Icon : OffIcon;
          return (
            <span
              className={cn(
                "flex min-w-0 flex-col gap-1 rounded-xl p-2",
                isOn ? "combined-raised" : "bg-(--fill)",
              )}
              key={id}
            >
              <Glyph className={cn("size-4", !isOn && "text-(--fg-3)")} />
              <span className={cn("text-[12px]", !isOn && "text-(--fg-2)")}>
                {label}
              </span>
              <span className="flex items-center gap-0.5 text-[10px] text-(--fg-3)">
                <span className="truncate">{isOn ? device : "Off"}</span>
                <ChevronDown className="size-3 shrink-0" />
              </span>
              <span className="h-[3px] overflow-hidden rounded-full bg-(--fill-2)">
                {isOn && id !== "camera" ? (
                  <span
                    className="block h-full rounded-full bg-(--level)"
                    style={{ width: id === "microphone" ? "62%" : "40%" }}
                  />
                ) : null}
              </span>
            </span>
          );
        })}
      </div>
      <div className="flex items-center justify-between">
        <span className="flex flex-col">
          <span className="text-[12px]">Replay buffer</span>
          <span className="text-[11px] text-(--fg-3)">
            Keeps the last 30 seconds
          </span>
        </span>
        <Toggle isOn={false} />
      </div>
      <div className="flex items-center gap-1.5">
        <Btn>Screenshot</Btn>
        <Btn className="grow justify-between" variant="primary">
          Record
          <span className="tracking-widest opacity-60">⇧⌘R</span>
        </Btn>
        <IconBtn label="Options">
          <SlidersHorizontal />
        </IconBtn>
      </div>
    </div>
  );
}
