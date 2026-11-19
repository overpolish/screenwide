// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

import { getProjectThumbnail } from "../../project-browser/api";

import {
  addCue,
  cuePreview,
  type CueRequest,
  takeCueRequest,
  TELEPROMPTER_CUE_REQUEST_EVENT,
} from "./cue-api";

/** A cue from an editor waiting for its word, with its picture to show
 * while it waits once drawn: a recording's moment, or a screenshot's
 * still's address. */
export type PlacingCue = {
  picture: ImageBitmap | string | null;
  request: CueRequest;
};

const report = (what: string) => (error: unknown) => {
  console.error(what, error);
};

const pictureOf = async ({ source, timing }: CueRequest) => {
  if (timing) return cuePreview(source, timing.sourceMs);
  const still = await getProjectThumbnail(source);
  return still ? convertFileSrc(still) : null;
};

/**
 * The cue an editor asked for, waiting in the teleprompter for a word to be
 * picked: taken as the teleprompter opens and each time another is asked
 * for. Placed, it is made there; Escape lets it go.
 */
export function useCuePlacing() {
  const [placing, setPlacing] = useState<PlacingCue | null>(null);
  // Mounted, checked by the ref rather than the effect's run: run twice, as
  // in development, the second run's take finds the request already taken
  // by the first, whose answer must still be kept.
  const mountedRef = useRef(false);

  useEffect(() => {
    mountedRef.current = true;
    const take = () => {
      takeCueRequest()
        .then((request) => {
          if (!mountedRef.current || !request) return;
          setPlacing({ picture: null, request });
          return pictureOf(request).then((picture) => {
            if (!mountedRef.current) return;
            setPlacing((current) =>
              current?.request === request ? { picture, request } : current,
            );
          });
        })
        .catch(report("Could not take the cue to place"));
    };
    take();
    const unlistening = listen(TELEPROMPTER_CUE_REQUEST_EVENT, take);
    return () => {
      mountedRef.current = false;
      void unlistening.then((unlisten) => {
        unlisten();
      });
    };
  }, []);

  // Escape lets the cue go before it reaches the teleprompter's own keys,
  // which would close the window.
  useEffect(() => {
    if (!placing) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopImmediatePropagation();
      setPlacing(null);
    };
    window.addEventListener("keydown", onKeyDown, true);
    return () => {
      window.removeEventListener("keydown", onKeyDown, true);
    };
  }, [placing]);

  return {
    /** Makes the cue waiting at the word starting `at` in the script
     * project `script`. */
    place: (at: number, script: string) => {
      if (!placing) return;
      setPlacing(null);
      addCue({ at, script, ...placing.request }).catch(
        report("Could not pin the cue"),
      );
    },
    placing,
  };
}
