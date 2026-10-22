// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Mic } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useRef, useState } from "react";

import { Switch } from "../../../primitives/switch/switch";
import { FakeScreen } from "../shared/picture";

import { IdeaFrame } from "./idea-frame";
import {
  CountdownContent,
  DoneContent,
  type Inputs,
  LiveContent,
  ReadyContent,
} from "./living-bar-phases";

// One shape through the whole recording. Ready, it shows what will be
// recorded and hears every input; Record folds it into a countdown; while
// recording it is a quiet dot and a time that open into controls under the
// pointer; when it stops it becomes the recording. Every change is the same
// shape moving, never one window swapped for another.

type Phase = "countdown" | "done" | "paused" | "ready" | "recording";

const morph = { bounce: 0.18, duration: 0.5, type: "spring" } as const;

export function LivingBar() {
  const [phase, setPhase] = useState<Phase>("ready");
  const [count, setCount] = useState(3);
  const [seconds, setSeconds] = useState(0);
  const [moments, setMoments] = useState<number[]>([]);
  const [inputs, setInputs] = useState<Inputs>({
    camera: true,
    mic: true,
    system: true,
  });
  const [isOpen, setIsOpen] = useState(false);
  const closeRef = useRef(0);
  const [isSilent, setIsSilent] = useState(false);

  useEffect(() => {
    if (phase === "countdown") {
      const timer = setTimeout(() => {
        if (count <= 1) setPhase("recording");
        else setCount(count - 1);
      }, 900);
      return () => {
        clearTimeout(timer);
      };
    }
    if (phase !== "recording") return;
    const timer = setInterval(() => {
      setSeconds((value) => value + 0.1);
    }, 100);
    return () => {
      clearInterval(timer);
    };
  }, [phase, count]);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key.toLowerCase() === "m" && phase === "recording") {
        setMoments((current) => [...current, seconds]);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, [phase, seconds]);

  const warns = phase === "recording" && isSilent && inputs.mic && seconds > 2;

  return (
    <IdeaFrame
      hint="Click an input to switch it off. Record, then hover the bar while it records; press M for a moment. Turn on the silent microphone to see how it warns."
      pitch="One shape through the whole recording. Its inputs show their own levels, it folds into the countdown, it stays a quiet breathing dot while you work and opens under the pointer, and when you stop it becomes the recording."
      title="A living recording bar"
    >
      <div className="flex items-center gap-separate self-start">
        <Switch isSelected={isSilent} onChange={setIsSilent}>
          Silent microphone
        </Switch>
      </div>
      <div className="relative h-[520px] w-[860px] overflow-hidden rounded-panel">
        <FakeScreen className="size-full" />
        <div className="absolute inset-x-0 bottom-10 flex flex-col items-center gap-2">
          <AnimatePresence>
            {warns ? (
              <motion.div
                animate={{ opacity: 1, y: 0 }}
                className="flex items-center gap-2 rounded-full bg-[#1c1c1e]/88 py-1.5 ps-3 pe-1.5 text-label text-white shadow-xl backdrop-blur"
                exit={{ opacity: 0, y: 6 }}
                initial={{ opacity: 0, y: 6 }}
              >
                <Mic className="size-4 text-error" />
                Your microphone hears nothing
                <button
                  className="rounded-full bg-white/15 px-2.5 py-1 hover:bg-white/25"
                  onClick={() => {
                    setIsSilent(false);
                  }}
                  type="button"
                >
                  Choose another
                </button>
              </motion.div>
            ) : null}
          </AnimatePresence>
          {/* Hover is read from a still frame around the bar, not the bar
              itself, which moves while it changes shape; and it closes a
              moment after the pointer leaves, so a slip does not slam it
              shut. */}
          <div
            className="p-3"
            onPointerEnter={() => {
              window.clearTimeout(closeRef.current);
              setIsOpen(true);
            }}
            onPointerLeave={() => {
              closeRef.current = window.setTimeout(() => {
                setIsOpen(false);
              }, 350);
            }}
          >
            <motion.div
              className="overflow-hidden bg-[#1c1c1e]/88 text-white shadow-2xl backdrop-blur-xl"
              layout
              style={{ borderRadius: phase === "done" ? 18 : 26 }}
              transition={morph}
            >
              <AnimatePresence initial={false} mode="popLayout">
                {phase === "ready" ? (
                  <ReadyContent
                    inputs={inputs}
                    isSilent={isSilent}
                    key="ready"
                    onRecord={() => {
                      setCount(3);
                      setSeconds(0);
                      setMoments([]);
                      setPhase("countdown");
                    }}
                    onToggle={(input) => {
                      setInputs((current) => ({
                        ...current,
                        [input]: !current[input],
                      }));
                    }}
                  />
                ) : phase === "countdown" ? (
                  <CountdownContent
                    count={count}
                    key="countdown"
                    onCancel={() => {
                      setPhase("ready");
                    }}
                  />
                ) : phase === "done" ? (
                  <DoneContent
                    key="done"
                    moments={moments.length}
                    onAgain={() => {
                      setPhase("ready");
                    }}
                    seconds={seconds}
                  />
                ) : (
                  <LiveContent
                    isPaused={phase === "paused"}
                    isSilent={isSilent || !inputs.mic}
                    key="live"
                    moments={moments}
                    onDiscard={() => {
                      setPhase("ready");
                    }}
                    onMoment={() => {
                      setMoments((current) => [...current, seconds]);
                    }}
                    onPause={() => {
                      setPhase(phase === "paused" ? "recording" : "paused");
                    }}
                    onStop={() => {
                      setPhase("done");
                    }}
                    seconds={seconds}
                    showsControls={phase === "paused" || isOpen}
                  />
                )}
              </AnimatePresence>
            </motion.div>
          </div>
        </div>
      </div>
    </IdeaFrame>
  );
}
