// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type CSSProperties, type ReactNode } from "react";

import { cn } from "../../../lib/styling";

import { boxStyle, type Element, ELEMENTS } from "./ideas-data";

// The recorded screen: an app drawn from the same elements a pointer snaps
// to, so what is seen is what is found. Content, so the same in every
// direction, and in the app's own light look whatever the editor's theme.

const LOOK: Record<Element["kind"], string> = {
  button: "bg-[#f2f2f4] text-[#1d1d1f]",
  card: "items-end justify-start bg-[#f7f7f9] p-[6%] text-[#1d1d1f] shadow-[0_1px_2px_rgba(0,0,0,0.08)]",
  field: "justify-start bg-[#f2f2f4] px-[3%] text-[#8e8e93]",
  nav: "justify-start px-[6%] text-[#3a3a3c]",
  primary: "bg-[#1d1d1f] text-white",
  title: "justify-start text-[15px] font-semibold text-[#1d1d1f]",
};

// Each card's picture: three template layouts, sketched.
const CARD_ART: Record<string, string> = {
  blank: "",
  gallery: "grid grid-cols-3 gap-[6%]",
  starter: "grid grid-cols-2 gap-[6%]",
};

function CardArt({ id }: { id: string }) {
  const layout = CARD_ART[id];
  if (!layout) return null;
  const tiles = id === "gallery" ? 6 : 4;
  return (
    <span className={cn("absolute inset-x-[8%] top-[8%] h-[58%]", layout)}>
      {Array.from({ length: tiles }, (_, index) => (
        <span
          className="rounded-[4px]"
          key={index}
          style={{
            background: ["#e8f0ff", "#fff1e6", "#eafbf0", "#f5e9ff"][index % 4],
          }}
        />
      ))}
    </span>
  );
}

export function MockApp({
  children,
  className,
  style,
}: {
  children?: ReactNode;
  className?: string;
  style?: CSSProperties;
}) {
  return (
    <div
      className={cn(
        "relative aspect-[16/10] overflow-hidden bg-white text-[12px] select-none",
        className,
      )}
      dir="ltr"
      style={style}
    >
      <span className="absolute inset-x-0 top-0 h-[9.5%] border-b border-[#ececf0] bg-[#fbfbfc]" />
      <span className="absolute top-[9.5%] bottom-0 w-[22%] border-e border-[#ececf0] bg-[#f7f7f9]" />
      <span className="absolute top-[3.6%] flex gap-[5px] ps-[1.6%]">
        {["#ff5f57", "#febc2e", "#28c840"].map((color) => (
          <span
            className="size-[10px] rounded-full"
            key={color}
            style={{ background: color }}
          />
        ))}
      </span>
      {ELEMENTS.map((element) => (
        <span
          className={cn(
            "absolute flex items-center justify-center overflow-hidden font-medium",
            LOOK[element.kind],
          )}
          key={element.id}
          style={{ ...boxStyle(element), borderRadius: element.radius }}
        >
          {element.kind === "card" ? <CardArt id={element.id} /> : null}
          <span className="relative">{element.label}</span>
        </span>
      ))}
      {children}
    </div>
  );
}
