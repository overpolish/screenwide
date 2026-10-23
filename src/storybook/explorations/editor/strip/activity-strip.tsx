// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { animate, type AnimationPlaybackControls } from "motion/react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";

import { ACTIONS, RECORDING_S, STEPS, stepAt } from "../editor-data";
import { type Editor } from "../model/use-editor";

import { MarkRail, type Snap } from "./mark-rail";
import { BeatBlock, Seams, Stills, VoiceRow } from "./strip-beats";
import { Chapters, LensPreview, Playhead, SceneRail } from "./strip-rails";
import { type Lens, stripScale, type StripScale } from "./strip-scale";

// The timeline as one strip of what happened. Its length is the edited
// recording's, against the full recording's width, so squeezing or cutting
// visibly shortens it.
//
// The lens swells time under a slow pointer and flattens under a fast one,
// so it helps when aiming and stays out of the way when travelling; the
// moment under the pointer stays under it. Anything pressed freezes the lens
// until it is let go, so nothing moves while it is held. Pressing a beat
// selects it; scrubbing is from the step names along the top or the
// playhead's knob; pressing the empty strip moves the playhead there.

/** Pointer speed, in px per ms, past which the lens flattens and follows. */
const FAST = 0.6;
/** While aiming, how far the pointer may stray from the lens's middle, in
 * px, before the lens is drawn after it. */
const LEASH_PX = 56;
/** How close, in px, an annotation's end must come to an event to snap. */
const SNAP_PX = 8;

function useWidth() {
  const ref = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(0);
  useLayoutEffect(() => {
    const element = ref.current;
    if (!element) return;
    const observer = new ResizeObserver(([entry]) => {
      setWidth(entry.contentRect.width);
    });
    observer.observe(element);
    return () => {
      observer.disconnect();
    };
  }, []);
  return { ref, width };
}

/** The lens: where it is and how strong, moved by the pointer's speed, and
 * held still while anything on the strip is pressed. */
function useLens() {
  const [lens, setLens] = useState<Lens>({ focus: 0, strength: 0 });
  const motionRef = useRef<{
    controls: AnimationPlaybackControls | null;
    focus: number;
    last: { t: number; x: number } | null;
    strength: number;
    target: number;
  }>({ controls: null, focus: 0, last: null, strength: 0, target: 0 });
  const isHeldRef = useRef(false);
  useEffect(() => {
    const release = () => {
      isHeldRef.current = false;
    };
    window.addEventListener("pointerup", release);
    return () => {
      window.removeEventListener("pointerup", release);
    };
  }, []);
  const swell = (target: number) => {
    const state = motionRef.current;
    if (state.target === target) return;
    state.target = target;
    state.controls?.stop();
    state.controls = animate(state.strength, target, {
      duration: 0.22,
      onUpdate: (strength) => {
        state.strength = strength;
        setLens((now) => ({ ...now, strength }));
      },
    });
  };
  return {
    /** Scrubbing travels, so it reads a flat strip; the lens forms again
     * where the pointer is once it is let go and moves. */
    flatten: () => {
      const state = motionRef.current;
      state.controls?.stop();
      state.strength = 0;
      state.target = 0;
      isHeldRef.current = true;
      setLens((now) => ({ ...now, strength: 0 }));
    },
    hold: () => {
      isHeldRef.current = true;
    },
    leave: () => {
      motionRef.current.last = null;
      if (!isHeldRef.current) swell(0);
    },
    lens,
    // Travelling fast, the lens flattens and follows. Slowing, it forms where
    // the pointer is (its middle stays put, so nothing jumps) and then holds
    // still: the pointer moves over a magnified strip that is not moving, so
    // aiming is four times finer. Strayed past the leash, it is drawn along.
    move: (
      event: React.PointerEvent,
      { flat, width, x }: { flat: StripScale; width: number; x: number },
    ) => {
      if (isHeldRef.current) return;
      const state = motionRef.current;
      const now = event.timeStamp;
      const speed = state.last
        ? Math.abs(event.clientX - state.last.x) /
          Math.max(now - state.last.t, 1)
        : 0;
      state.last = { t: now, x: event.clientX };
      const isFast = speed > FAST;
      const isForming = state.target === 0;
      const stray = (x - flat.toX(state.focus)) * width;
      if (isFast || isForming) state.focus = flat.toSeconds(x);
      else if (Math.abs(stray) > LEASH_PX)
        state.focus = flat.toSeconds(x - (Math.sign(stray) * LEASH_PX) / width);
      const { focus } = state;
      setLens((current) => ({ ...current, focus }));
      swell(isFast ? 0 : 1);
    },
  };
}

export function ActivityStrip({ editor }: { editor: Editor }) {
  const area = useWidth();
  const track = useWidth();
  const { flatten, hold, leave, lens, move } = useLens();
  const gripRef = useRef<{ squeeze: number; x: number } | null>(null);
  const [isGripping, setIsGripping] = useState(false);
  const { map } = editor;

  const flat = stripScale({ map });
  const scale = stripScale({
    lens: isGripping ? undefined : lens,
    map,
  });
  const fractionOf = (event: React.PointerEvent) => {
    const box = track.ref.current?.getBoundingClientRect();
    return box ? (event.clientX - box.left) / box.width : 0;
  };
  const secondsAt = (event: React.PointerEvent) =>
    scale.toSeconds(fractionOf(event));
  const scrub = (event: React.PointerEvent) => {
    editor.seek(secondsAt(event));
  };
  const scrubFlat = (event: React.PointerEvent) => {
    if (event.type === "pointerdown") flatten();
    editor.seek(flat.toSeconds(fractionOf(event)));
  };
  const events = [
    ...STEPS.flatMap(({ end, start }) => [start, end]),
    ...editor.beats.flatMap(({ end, start }) => [start, end]),
    ...ACTIONS.map(({ at }) => at),
    editor.playhead,
  ];
  const snap: Snap = (seconds) => {
    const near = events.reduce((best, event) =>
      Math.abs(event - seconds) < Math.abs(best - seconds) ? event : best,
    );
    const isSnapped =
      Math.abs(scale.toX(near) - scale.toX(seconds)) * track.width < SNAP_PX;
    return { isSnapped, seconds: isSnapped ? near : seconds };
  };
  const selectedBeat =
    editor.selection?.kind === "beat" ? editor.selection.id : undefined;

  return (
    <div className="relative min-w-0 grow pe-5" ref={area.ref}>
      <div
        className="relative touch-none pt-5 pb-1"
        dir="ltr"
        onPointerDown={(event) => {
          scrub(event);
        }}
        onPointerDownCapture={hold}
        onPointerLeave={leave}
        onPointerMove={(event) => {
          move(event, { flat, width: track.width, x: fractionOf(event) });
        }}
        ref={track.ref}
        style={{ width: `${String((map.duration / RECORDING_S) * 100)}%` }}
      >
        <Chapters scale={scale} />
        {/* The step names double as the ruler: drag along them to scrub. */}
        <div
          className="absolute inset-x-0 top-0 z-10 h-5 cursor-ew-resize"
          onPointerDown={(event) => {
            event.stopPropagation();
            event.currentTarget.setPointerCapture(event.pointerId);
            scrubFlat(event);
          }}
          onPointerMove={(event) => {
            if (event.currentTarget.hasPointerCapture(event.pointerId))
              scrubFlat(event);
          }}
        />
        {editor.mode === "scene" ? (
          <div className="flex h-6 flex-col justify-center">
            <SceneRail
              current={stepAt(editor.playhead)}
              scale={scale}
              scenes={editor.scenes}
            />
          </div>
        ) : (
          <MarkRail
            marks={editor.kind === "recording" ? editor.marks : []}
            onRetime={editor.retimeMark}
            onSelect={(id) => {
              editor.select({ id, kind: "mark" });
            }}
            scale={scale}
            secondsAt={secondsAt}
            selected={
              editor.selection?.kind === "mark"
                ? editor.selection.id
                : undefined
            }
            snap={snap}
            width={track.width}
          />
        )}
        <div className="relative h-11">
          <Stills pieces={map.pieces} scale={scale} width={track.width} />
          {editor.beats.map((beat) => (
            <BeatBlock
              beat={beat}
              isSelected={beat.id === selectedBeat}
              key={beat.id}
              onPress={() => {
                editor.select({ id: beat.id, kind: "beat" });
                editor.seek(
                  beat.kind === "action" ? beat.action.at : beat.start,
                );
              }}
              scale={scale}
              width={track.width}
            />
          ))}
          <Seams cuts={editor.cuts} onRestore={editor.restore} scale={scale} />
        </div>
        {editor.isVoiced && editor.kind === "recording" ? (
          <div className="mt-1">
            <VoiceRow scale={scale} width={track.width} />
          </div>
        ) : null}
        <span className="pointer-events-none absolute inset-x-0 top-5 bottom-0">
          <Playhead
            onScrub={scrubFlat}
            scale={scale}
            seconds={editor.playhead}
          />
        </span>
        {lens.strength > 0.6 && !isGripping ? (
          <LensPreview scale={scale} seconds={lens.focus} />
        ) : null}
        {/* The end is the strip's length: drag it in to squeeze. */}
        <button
          aria-label="Squeeze"
          className="absolute -end-4 top-5 bottom-0 flex w-4 cursor-ew-resize items-center justify-center"
          onDoubleClick={() => {
            editor.setSqueeze(0);
          }}
          onPointerDown={(event) => {
            event.stopPropagation();
            event.currentTarget.setPointerCapture(event.pointerId);
            gripRef.current = { squeeze: editor.squeeze, x: event.clientX };
            setIsGripping(true);
          }}
          onPointerMove={(event) => {
            const grip = gripRef.current;
            if (!grip || area.width === 0) return;
            const seconds =
              ((grip.x - event.clientX) / area.width) * RECORDING_S;
            editor.setSqueeze(
              Math.min(Math.max(grip.squeeze + seconds, 0), map.maxSqueeze),
            );
          }}
          onPointerUp={() => {
            gripRef.current = null;
            setIsGripping(false);
          }}
          title="Drag in to squeeze the still time; double-click to undo"
          type="button"
        >
          <span className="h-10 w-1.5 rounded-full bg-fg-secondary" />
        </button>
      </div>
    </div>
  );
}
