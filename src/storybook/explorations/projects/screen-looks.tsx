// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../lib/styling";
import { Waveform } from "../shared/picture";

import { place, type Project } from "./projects-model";

// What each kind of recording shows, drawn at the recorded screen's full
// size: a web page, code, an app's settings, slides, a talk to camera,
// sound. Each step draws its own state (the page scrolled on, a file
// opened, the next slide) and the boxes match the steps' outlines in the
// library. Drawn inside a left-to-right root, so start is the screen's left.

function Bars({
  className,
  gap = 5,
  rows,
  x,
  y,
}: {
  className: string;
  rows: number[];
  x: number;
  y: number;
  gap?: number;
}) {
  return (
    <>
      {rows.map((width, index) => (
        <span
          className={cn("absolute h-[2.2%] rounded-full", className)}
          key={width}
          style={place({ h: 2.2, w: width, x, y: y + index * gap })}
        />
      ))}
    </>
  );
}

function Chrome({ dark = false }: { dark?: boolean }) {
  return (
    <span
      className={cn(
        "absolute inset-x-0 top-0 flex h-[9%] items-center gap-[5px] ps-[1.6%]",
        dark ? "bg-[#16181d]" : "bg-[#fbfbfc]",
      )}
    >
      {["#ff5f57", "#febc2e", "#28c840"].map((color) => (
        <span
          className="size-[10px] rounded-full"
          key={color}
          style={{ background: color }}
        />
      ))}
    </span>
  );
}

export function Browser({ project, step }: { project: Project; step: number }) {
  const isCheckout = step >= 3;
  // Each page is the pricing page's layout, under its own heading.
  const isPricing = project.id === "pricing";
  return (
    <div className="absolute inset-0 bg-white">
      <Chrome />
      <span className="absolute inset-x-0 top-[9%] h-[13%] border-b border-[#ececf0] bg-[#f7f7f9]" />
      <span
        className="absolute flex items-center rounded-[8px] bg-white px-[1.5%] text-[13px] text-[#6e6e73] shadow-[0_0_0_1px_#e5e5ea]"
        style={place({ h: 7, w: 54, x: 23, y: 11 })}
      >
        {`screenwide.app/${isCheckout ? "checkout" : isPricing ? "pricing" : "start"}`}
      </span>
      {isCheckout ? (
        <>
          <span className="absolute start-[30%] top-[30%] text-[22px] font-semibold text-[#1d1d1f]">
            Checkout
          </span>
          {[42, 54, 66].map((top) => (
            <span
              className="absolute rounded-[8px] bg-[#f2f2f4]"
              key={top}
              style={place({ h: 8, w: 40, x: 30, y: top })}
            />
          ))}
          <span
            className="absolute flex items-center justify-center rounded-[8px] bg-[#1d1d1f] text-[13px] text-white"
            style={place({ h: 8, w: 40, x: 30, y: 80 })}
          >
            Pay
          </span>
        </>
      ) : (
        <>
          <span className="absolute inset-x-0 top-[27%] text-center text-[24px] font-semibold text-[#1d1d1f]">
            {isPricing ? "Simple pricing" : project.title}
          </span>
          <Bars className="bg-[#e5e5ea]" rows={[30]} x={35} y={35} />
          {["Solo", "Team", "Studio"].map((plan, index) => (
            <span
              className={cn(
                "absolute flex flex-col gap-[6%] rounded-[12px] p-[2%] font-semibold",
                index === 1
                  ? "bg-[#1d1d1f] text-white"
                  : "bg-[#f7f7f9] text-[#1d1d1f] shadow-[0_0_0_1px_#ececf0]",
              )}
              key={plan}
              style={place({ h: 40, w: 26, x: 7 + index * 30, y: 42 })}
            >
              <span className="text-[14px]">{plan}</span>
              <span className="text-[22px]">{`$${String([0, 12, 29][index])}`}</span>
            </span>
          ))}
          <span
            className="absolute flex items-center justify-center rounded-[8px] bg-[#ff7a3d] text-[12px] font-medium text-white"
            style={place({ h: 8, w: 18, x: 41, y: 74 })}
          >
            Start free trial
          </span>
        </>
      )}
    </div>
  );
}

export function Code({ step }: { step: number }) {
  const files = ["app.tsx", "routes.ts", "session.ts", "login.tsx", "api.ts"];
  return (
    <div className="absolute inset-0 bg-[#1c1f26] font-mono">
      <Chrome dark />
      <span className="absolute start-0 top-[9%] bottom-0 w-[20%] bg-[#16181d]" />
      {files.map((file, index) => (
        <span
          className={cn(
            "absolute flex items-center rounded-[5px] px-[0.8%] text-[11px]",
            step >= 1 && index === 2
              ? "bg-[#2d3340] text-white"
              : "text-[#8b93a5]",
          )}
          key={file}
          style={place({ h: 6, w: 15, x: 2, y: 15 + index * 8 })}
        >
          {file}
        </span>
      ))}
      <Bars
        className="bg-[#3a3f4b]"
        gap={6}
        rows={[31, 43, 21, 49, 36, 27, 46, 17, 41]}
        x={26}
        y={14}
      />
      {step >= 2 ? (
        <span
          className="absolute rounded-[4px] bg-[#28c84033]"
          style={place({ h: 6, w: 52, x: 26, y: 40 })}
        />
      ) : null}
      {step === 0 || step === 3 ? (
        <span
          className="absolute flex flex-col gap-[8%] border-t border-[#2d3340] bg-[#16181d] p-[1.2%] text-[11px]"
          style={place({ h: 22, w: 78, x: 22, y: 78 })}
        >
          <span className="text-[#8b93a5]">
            {step === 0 ? "$ npm run dev" : "$ npm test"}
          </span>
          <span className={step === 0 ? "text-[#ff6b6b]" : "text-[#4ade80]"}>
            {step === 0 ? "401 session expired" : "✓ 48 passed"}
          </span>
        </span>
      ) : null}
    </div>
  );
}

export function Design({ step }: { step: number }) {
  return (
    <div className="absolute inset-0 bg-[#fbfdfc]">
      <Chrome />
      <span className="absolute start-0 top-[9%] bottom-0 w-[23%] bg-[#eff5f2]" />
      {["General", "Tools", "Moments", "Shortcuts"].map((item, index) => (
        <span
          className={cn(
            "absolute flex items-center rounded-[6px] px-[1%] text-[12px]",
            index === step ? "bg-[#1f9d6f] text-white" : "text-[#3a3a3c]",
          )}
          key={item}
          style={place({ h: 7, w: 18, x: 3, y: 18 + index * 9 })}
        >
          {item}
        </span>
      ))}
      <span
        className="absolute rounded-[12px] bg-gradient-to-br from-[#f59e0b] to-[#ec4899]"
        style={place({ h: 34, w: 70, x: 26, y: 18 })}
      >
        {step >= 1
          ? [10, 52].map((x) => (
              <span
                className="absolute rounded-[8px] bg-white/90"
                key={x}
                style={place({ h: 64, w: 38, x, y: 18 })}
              />
            ))
          : null}
      </span>
      <Bars
        className="bg-[#e2ebe7]"
        gap={8}
        rows={[50, 40, 56]}
        x={26}
        y={60}
      />
    </div>
  );
}

export function Slides({ project, step }: { project: Project; step: number }) {
  return (
    <div className="absolute inset-0 bg-[#2a2a2e]">
      {project.steps.map(({ title }, index) => (
        <span
          className={cn(
            "absolute rounded-[3px] bg-white",
            index === step && "outline-2 outline-[#f59e0b]",
          )}
          key={title}
          style={place({ h: 13, w: 13, x: 2, y: 6 + index * 17 })}
        />
      ))}
      <span
        className="absolute flex flex-col justify-end gap-[3%] rounded-[4px] bg-gradient-to-br from-[#fff7e6] to-[#fde68a] p-[5%]"
        style={place({ h: 84, w: 78, x: 18, y: 8 })}
      >
        <span className="text-[13px] font-medium tracking-wide text-[#b45309] uppercase">
          {`Q3 · ${String(step + 1)}`}
        </span>
        <span className="text-[38px] leading-none font-semibold text-[#1c1917]">
          {project.steps[step]?.title}
        </span>
      </span>
    </div>
  );
}

export function Camera({ project }: { project: Project }) {
  const [light, dark] = project.tint;
  return (
    <div
      className="absolute inset-0"
      style={{
        background: `radial-gradient(circle at 30% 30%, ${light}, ${dark})`,
      }}
    >
      <span
        className="absolute rounded-[6px] bg-[#3d2f25]/40"
        style={place({ h: 40, w: 18, x: 72, y: 12 })}
      />
      <span
        className="absolute rounded-full bg-[#2a211b]"
        style={place({ h: 32, w: 20, x: 40, y: 20 })}
      />
      <span
        className="absolute rounded-t-full bg-[#2a211b]"
        style={place({ h: 44, w: 48, x: 26, y: 56 })}
      />
    </div>
  );
}

export function Audio({ project, step }: { project: Project; step: number }) {
  return (
    <div className="absolute inset-0 flex flex-col items-center justify-center gap-[6%] bg-gradient-to-br from-[#2b2d42] to-[#111111] text-white/80">
      <Waveform bars={90} className="h-[36%] w-[80%]" seed={step + 3} />
      <span className="text-[16px] font-medium">
        {project.steps[step]?.title}
      </span>
    </div>
  );
}
