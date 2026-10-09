// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

type Choice = "off" | "on";

type Held = {
  artifactId: number;
  /** Whether the app is still carrying out the last turn of the switch. */
  isPending: boolean;
  state: Choice;
};

/**
 * A microphone switch the app carries out on a file of its own, as Reduce
 * noise and Vocal cleanup are: read from the project when the recording
 * opens, and turned at once, the app's answer settling it.
 */
export function useMicrophoneSwitch({
  artifactId,
  name,
  read,
  write,
}: {
  artifactId: number | null;
  /** What the switch does, for the console when the app cannot answer. */
  name: string;
  read: (artifactId: number) => Promise<Choice>;
  write: (artifactId: number, enabled: boolean) => Promise<Choice>;
}) {
  const [held, setHeld] = useState<Held | null>(null);

  useEffect(() => {
    if (artifactId === null) return;
    let current = true;
    read(artifactId)
      .then((state) => {
        if (current) setHeld({ artifactId, isPending: false, state });
      })
      .catch((cause: unknown) => {
        console.error(`Could not read the microphone's ${name} choice`, cause);
      });
    return () => {
      current = false;
    };
  }, [artifactId, name, read]);

  const isCurrent = artifactId !== null && held?.artifactId === artifactId;
  const state: Choice = isCurrent ? held.state : "off";
  return {
    isPending: isCurrent && held.isPending,
    state,
    turn: (enabled: boolean) => {
      if (artifactId === null) return;
      setHeld({
        artifactId,
        isPending: enabled,
        state: enabled ? "on" : "off",
      });
      write(artifactId, enabled)
        .then((next) => {
          setHeld({ artifactId, isPending: false, state: next });
        })
        .catch((cause: unknown) => {
          console.error(`Could not change the microphone's ${name}`, cause);
          setHeld({ artifactId, isPending: false, state });
        });
    },
  };
}
