// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check } from "lucide-react";
import { useEffect } from "react";
import { createPortal } from "react-dom";

import { OverflowShadow } from "../../../components/base/overflow-shadow/overflow-shadow";

const TIMELINE_SEGMENT_PLAYBACK_RATES = [0.5, 0.75, 1, 1.25, 1.5, 2] as const;

export type TimelineSpeedMenuState = {
  x: number;
  y: number;
};

export function TimelineSegmentSpeedContextMenu({
  menu,
  onChange,
  onClose,
  playbackRate,
  title,
}: {
  menu: TimelineSpeedMenuState;
  onChange: (playbackRate: number) => void;
  onClose: () => void;
  title: string;
  playbackRate?: number;
}) {
  useEffect(() => {
    window.addEventListener("blur", onClose);
    window.addEventListener("pointerdown", onClose);
    return () => {
      window.removeEventListener("blur", onClose);
      window.removeEventListener("pointerdown", onClose);
    };
  }, [onClose]);

  return createPortal(
    <div
      className="fixed z-50 w-28 max-h-[inherit]"
      onContextMenu={(event) => {
        event.preventDefault();
      }}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
      style={{
        left: menu.x,
        maxHeight: `calc(100vh - ${(menu.y + 8).toString()}px)`,
        top: menu.y,
      }}
    >
      <OverflowShadow
        constrainHeight
        rootClassName="max-h-[inherit] border-1 border-muted/25 bg-content shadow-lg"
      >
        <div aria-label={`${title} playback speed`} className="p-1" role="menu">
          <p className="m-0 px-2 py-1 text-[10px] font-medium text-muted">
            {title} speed
          </p>
          {TIMELINE_SEGMENT_PLAYBACK_RATES.map((rate) => {
            const selected = rate === playbackRate;
            const label = `${rate.toString()}×`;
            return (
              <button
                aria-checked={selected}
                className="flex w-full items-center rounded px-2 py-1.5 text-left text-xs text-content-fg outline-none transition-colors hover:bg-muted/10"
                key={rate}
                onClick={() => {
                  onChange(rate);
                  onClose();
                }}
                role="menuitemradio"
                type="button"
              >
                <span className="grow">{label}</span>
                {selected ? <Check className="text-success" size={14} /> : null}
              </button>
            );
          })}
        </div>
      </OverflowShadow>
    </div>,
    document.body,
  );
}
