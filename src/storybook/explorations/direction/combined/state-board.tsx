// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import "./combined.css";

import {
  ArrowUpRight,
  Brackets,
  Captions,
  FolderOpen,
  Magnet,
  Scissors,
  Trash2,
} from "lucide-react";
import { type ReactNode } from "react";

import { cn } from "../../../../lib/styling";
import { Chip } from "../../export/export-parts";

import { Btn, IconBtn, Row, Segmented, Toggle } from "./combined-controls";

// Where each colour role shows, each sample drawn as the mocks draw it: the
// primary action, what is checked or on, what is chosen in place, and the
// orange kept for emphasis.

function Card({
  children,
  how,
  title,
}: {
  children: ReactNode;
  how: string;
  title: string;
}) {
  return (
    <div className="flex flex-col gap-2 rounded-xl bg-(--fill) p-3">
      <span className="text-[12px] font-medium">{title}</span>
      <div className="flex min-h-12 items-center gap-2">{children}</div>
      <span className="text-[11px] text-(--fg-3)">{how}</span>
    </div>
  );
}

function Section({ children, title }: { children: ReactNode; title: string }) {
  return (
    <section className="flex flex-col gap-2">
      <span className="text-[13px] font-semibold">{title}</span>
      <div className="grid grid-cols-3 gap-2">{children}</div>
    </section>
  );
}

function OnCards() {
  return (
    <>
      <Card how="Filled track, the knob on it" title="Switch on">
        <Toggle isOn />
        <Toggle isOn={false} />
      </Card>
      <Card how="The filled part of the track" title="Slider">
        <span
          className="relative h-1 w-28 rounded-full bg-(--fill-2)"
          dir="ltr"
        >
          <span className="absolute inset-y-0 start-0 w-[45%] rounded-full bg-(--accent)" />
          <span className="combined-knob absolute top-1/2 start-[45%] size-3.5 -translate-1/2 rounded-full" />
        </span>
      </Card>
      <Card how="A dot that resets the row" title="Changed from default">
        <div className="w-40">
          <Row isChanged label="Width">
            <span />
          </Row>
        </div>
      </Card>
      <Card how="Filled, the label on it" title="Option on">
        <Chip icon={Captions} isOn>
          Subtitles
        </Chip>
        <Chip icon={Trash2}>Delete</Chip>
      </Card>
      <Card how="Filled: a switch that stays on" title="Option switched on">
        <IconBtn className="combined-accent" label="Snap to edges">
          <Magnet />
        </IconBtn>
        <span className="text-[11px] text-(--fg-3)">Snap</span>
      </Card>
      <Card how="Filled, the label on it" title="Selected clip">
        <span className="combined-accent flex h-[18px] items-center rounded-[5px] px-1.5 text-[11px]">
          Arrow 1
        </span>
        <span className="flex h-[18px] items-center rounded-[5px] bg-(--clip-annotation) px-1.5 text-[11px]">
          Text 1
        </span>
      </Card>
      <Card how="Filled, the icon on it" title="Current tool in the ring">
        <span className="combined-accent flex size-9 items-center justify-center rounded-full">
          <ArrowUpRight className="size-4" />
        </span>
        <span className="flex size-9 items-center justify-center rounded-full text-(--fg-2)">
          <FolderOpen className="size-4" />
        </span>
      </Card>
      <Card how="A ring around the swatch" title="Colour chosen">
        {["#ff3b30", "#ff9500", "#34c759"].map((color, index) => (
          <span
            className={cn(
              "size-[22px] rounded-md",
              index === 0 &&
                "ring-2 ring-(--accent) ring-offset-2 ring-offset-(--fill)",
            )}
            key={color}
            style={{ background: color }}
          />
        ))}
      </Card>
    </>
  );
}

function ChosenCards() {
  return (
    <>
      <Card how="The chosen segment is raised" title="Segmented choice">
        <Segmented
          className="w-36"
          items={["None", "End", "Both"]}
          selected={1}
        />
      </Card>
      <Card how="A grey surface, the text unchanged" title="Chosen style">
        {["Callout", "Subtle"].map((name, index) => (
          <span
            className={cn(
              "flex flex-col items-center gap-1 rounded-lg p-1 text-[10px] text-(--fg-2)",
              index === 0 && "combined-selected text-(--fg)",
            )}
            key={name}
          >
            <span className="flex size-9 items-center justify-center rounded-lg bg-[#1c1c1e] text-[#ff5a4f]">
              <ArrowUpRight className="size-5" />
            </span>
            {name}
          </span>
        ))}
      </Card>
      <Card
        how="A grey surface, the text unchanged"
        title="Words chosen to cut"
      >
        <span className="flex text-[11px] text-(--fg-2)">
          <span className="px-0.5">Name</span>
          <span className="px-0.5">it</span>
          <span className="combined-selected rounded-[3px] px-0.5">
            and pick
          </span>
        </span>
      </Card>
      <Card how="The tool in hand is raised" title="Tool in use">
        <IconBtn isActive label="Range">
          <Brackets />
        </IconBtn>
        <IconBtn label="Blade">
          <Scissors />
        </IconBtn>
      </Card>
    </>
  );
}

function EmphasisCards() {
  return (
    <>
      <Card how="The line and its knob" title="Playhead">
        <span className="relative h-10 w-40 rounded-md bg-(--fill-2)" dir="ltr">
          <span className="absolute inset-y-0 start-[40%] w-0.5 -translate-x-1/2 bg-(--emphasis)" />
          <span className="absolute -top-1 start-[40%] size-2.5 -translate-x-1/2 rounded-full bg-(--emphasis)" />
        </span>
      </Card>
      <Card how="Handles on what is selected" title="Selection on the picture">
        <span className="relative h-10 w-28 rounded-md bg-linear-to-br from-[#f6d365] to-[#fda085]">
          {["start-3 top-3", "end-4 bottom-2"].map((place) => (
            <span
              className={cn(
                "absolute size-3 rounded-full border-2 border-(--emphasis) bg-white",
                place,
              )}
              key={place}
            />
          ))}
        </span>
      </Card>
      <Card how="How far it has got" title="Progress">
        <span className="relative h-1 w-28 rounded-full bg-(--fill-2)">
          <span className="absolute inset-y-0 start-0 w-[42%] rounded-full bg-(--emphasis)" />
        </span>
      </Card>
      <Card how="A ring around the focused control" title="Keyboard focus">
        {/* An outline, since the raised material owns the box shadow here;
            the app draws the same ring as a shadow over its bevel. */}
        <Btn className="outline-3 outline-(--emphasis)/50">Cancel</Btn>
      </Card>
    </>
  );
}

export function StateBoard() {
  return (
    <div className="look-combined flex w-[880px] flex-col gap-5 p-6 font-sans text-(--fg)">
      <Section title="Primary action: near-black, near-white in dark">
        <Card how="The one thing the window is for" title="Primary action">
          <Btn>Cancel</Btn>
          <Btn variant="primary">Export</Btn>
        </Card>
      </Section>
      <Section title="Checked or on: the same fill">
        <OnCards />
      </Section>
      <Section title="Chosen in place: grey">
        <ChosenCards />
      </Section>
      <Section title="Emphasis: orange, never text">
        <EmphasisCards />
      </Section>
    </div>
  );
}
