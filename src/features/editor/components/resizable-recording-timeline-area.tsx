// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ReactNode,
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

import {
  ReportBlockHeight,
  ReportBlockHeightContext,
} from "./timeline-band-block";
import {
  clampTimelineHeight,
  fitTimelineHeight,
  TIMELINE_MIN_HEIGHT_PX,
  TIMELINE_PLAYBACK_ROW_HEIGHT_PX,
  timelineMaximumHeight,
} from "./timeline-band-metrics";
import {
  RegisterPlaybackRow,
  RegisterPlaybackRowContext,
} from "./timeline-band-playback-row";

const MIN_PREVIEW_HEIGHT = 160;
const INITIAL_HEIGHT = 200;

export function ResizableRecordingTimelineArea({
  artifactId,
  children,
  header,
  ready = true,
}: {
  artifactId: number;
  /** The timeline block: the ruler, the rows that scroll under it, and the
   * meter beside them. */
  children: ReactNode;
  /** The transport row, which stands above the scroller and never scrolls. */
  header: ReactNode;
  ready?: boolean;
}) {
  return (
    <TimelineArea header={header} key={artifactId} ready={ready}>
      {children}
    </TimelineArea>
  );
}

function TimelineArea({
  children,
  header,
  ready,
}: {
  children: ReactNode;
  header: ReactNode;
  ready: boolean;
}) {
  const rootRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const initializedRef = useRef(false);
  const dragRef = useRef<{ height: number; y: number } | null>(null);
  const [height, setHeight] = useState(INITIAL_HEIGHT);
  // Two stops, measured separately: what the preview can spare above the band,
  // and how tall the timeline block inside it actually is.
  const [spaceHeight, setSpaceHeight] = useState(520);
  const [measuredHeight, setMeasuredHeight] = useState(0);
  // A block that scrolls inside itself reports the height it wants, since a
  // clipped subtree cannot be read off; anything else is measured.
  const [reportedHeight, setReportedHeight] = useState<number | null>(null);
  const reportBlockHeight = useCallback<ReportBlockHeight>((height) => {
    setReportedHeight(height);
  }, []);
  const contentHeight = reportedHeight ?? measuredHeight;
  // The transport row is measured rather than assumed: a control can grow it,
  // and the band's height is that row plus the block under it.
  const playbackObserverRef = useRef<ResizeObserver | null>(null);
  const [playbackRowHeight, setPlaybackRowHeight] = useState(
    TIMELINE_PLAYBACK_ROW_HEIGHT_PX,
  );
  const registerPlaybackRow = useCallback<RegisterPlaybackRow>((element) => {
    playbackObserverRef.current?.disconnect();
    playbackObserverRef.current = null;
    if (!element) return;
    const measure = () => {
      setPlaybackRowHeight(element.offsetHeight);
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    playbackObserverRef.current = observer;
  }, []);
  const maximum = timelineMaximumHeight({
    contentHeight,
    headerHeight: playbackRowHeight,
    spaceHeight,
  });
  const clamp = (value: number) => clampTimelineHeight(value, maximum);
  const visibleHeight = clamp(height);
  useLayoutEffect(() => {
    const parent = rootRef.current?.parentElement;
    if (!parent) return;
    const measure = () => {
      setSpaceHeight(
        Math.max(
          TIMELINE_MIN_HEIGHT_PX,
          parent.clientHeight - MIN_PREVIEW_HEIGHT,
        ),
      );
    };
    const observer = new ResizeObserver(measure);
    observer.observe(parent);
    return () => {
      observer.disconnect();
    };
  }, []);
  // The block's own height is the band's other stop, and it moves: a track
  // added or a lane grown changes what "the whole timeline" means.
  useLayoutEffect(() => {
    const content = contentRef.current;
    if (!content) return;
    // No eager call: observing reports the block's size straight away, the
    // way the space stop above is measured.
    const observer = new ResizeObserver(() => {
      setMeasuredHeight(content.scrollHeight);
    });
    observer.observe(content);
    return () => {
      observer.disconnect();
    };
  }, []);
  /** Size the band to the whole timeline block, as far as the stops allow. */
  const fit = () => {
    if (!contentRef.current) return;
    setHeight(
      fitTimelineHeight({
        contentHeight: reportedHeight ?? contentRef.current.scrollHeight,
        headerHeight: playbackRowHeight,
        maximum,
      }),
    );
  };
  const fitRef = useRef(fit);
  fitRef.current = fit;
  // The first fit waits for the block's height to be known: a block that
  // scrolls inside itself reports it a render after it mounts, and fitting to
  // an unknown block would only fit the band to itself.
  useLayoutEffect(() => {
    if (!ready || initializedRef.current || contentHeight <= 0) return;
    initializedRef.current = true;
    fitRef.current();
  }, [contentHeight, ready]);
  const cancel = () => {
    if (!dragRef.current) return;
    setHeight(dragRef.current.height);
    dragRef.current = null;
  };
  const cancelRef = useRef(cancel);
  cancelRef.current = cancel;
  useEffect(() => {
    const blur = () => {
      cancelRef.current();
    };
    const escape = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || !dragRef.current) return;
      event.preventDefault();
      event.stopImmediatePropagation();
      blur();
    };
    window.addEventListener("keydown", escape, true);
    window.addEventListener("blur", blur);
    return () => {
      window.removeEventListener("keydown", escape, true);
      window.removeEventListener("blur", blur);
    };
  }, []);
  const update = (y: number) => {
    if (dragRef.current)
      setHeight(clamp(dragRef.current.height + dragRef.current.y - y));
  };
  const updateRef = useRef(update);
  updateRef.current = update;
  // The pointer is followed on the window rather than through the handle's own
  // pointer capture: the handle is a few pixels tall and slides under the
  // pointer as the band grows, so tracking that depends on it staying the
  // capture target loses the drag the moment the edge moves past the pointer.
  const releaseRef = useRef<(() => void) | null>(null);
  const beginDrag = (y: number) => {
    releaseRef.current?.();
    dragRef.current = { height: visibleHeight, y };
    const move = (event: PointerEvent) => {
      updateRef.current(event.clientY);
    };
    const finish = (event: PointerEvent) => {
      updateRef.current(event.clientY);
      dragRef.current = null;
      releaseRef.current?.();
    };
    const stop = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", finish);
      window.removeEventListener("pointercancel", abandon);
      releaseRef.current = null;
    };
    function abandon() {
      cancelRef.current();
      stop();
    }
    releaseRef.current = stop;
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", finish);
    window.addEventListener("pointercancel", abandon);
  };
  useEffect(() => () => releaseRef.current?.(), []);
  return (
    <div
      className="relative flex shrink-0 flex-col bg-fill-quaternary"
      ref={rootRef}
      style={{ height: visibleHeight }}
    >
      <div
        aria-label="Resize timeline"
        aria-orientation="horizontal"
        aria-valuemax={maximum}
        aria-valuemin={TIMELINE_MIN_HEIGHT_PX}
        aria-valuenow={visibleHeight}
        // The band's own top edge is the handle. The strip sits wholly inside
        // the band rather than straddling the edge: above it is the native
        // compositor's own view, which takes the pointer before the webview
        // ever sees it, so anything reaching over the edge would be dead. It
        // starts a couple of pixels in as well, because a pointer resting on
        // the compositor can still share the edge's own row at fractional
        // device pixels, which lit the handle from outside the band. These are
        // hit targets rather than measurements, which is why they are not
        // spacing tokens - the strip takes no layout height either way.
        // `outline-none` kills the engine's own focus ring as well as ours:
        // the press focuses the strip so the arrow keys carry on from where
        // the drag left off, and a ring drawn around the band's whole edge
        // reads as a border rather than as focus.
        className="group absolute inset-x-0 top-[2px] z-40 h-[6px] cursor-row-resize touch-none outline-none"
        onClick={(event) => {
          // A double click refits the band. Taken from `click` and its click
          // count rather than from `dblclick`: the drag's `preventDefault` on
          // pointerdown suppresses the compatibility mouse events, and `click`
          // is the one the spec still guarantees. It arrives after the second
          // pointerup, so the drag that press opened is already closed - this
          // only drops whatever a stray move or cancel might have left.
          if (event.detail < 2) return;
          dragRef.current = null;
          fit();
        }}
        onKeyDown={(event) => {
          if (event.key !== "ArrowUp" && event.key !== "ArrowDown") return;
          event.preventDefault();
          event.stopPropagation();
          setHeight(
            clamp(
              visibleHeight +
                (event.key === "ArrowUp" ? 1 : -1) * (event.shiftKey ? 48 : 24),
            ),
          );
        }}
        onPointerDown={(event) => {
          if (event.button !== 0) return;
          event.preventDefault();
          event.currentTarget.focus();
          beginDrag(event.clientY);
        }}
        role="separator"
        tabIndex={0}
      >
        {/* Nothing at rest: the band already reads as its own surface, so the
            edge only shows a line while it is under the pointer or held. It is
            drawn on the band's first row of pixels, where the edge is, which
            is above the strip that answers the pointer. */}
        <div className="absolute inset-x-0 top-[-2px] h-px bg-transparent group-hover:bg-primary" />
      </div>
      <RegisterPlaybackRowContext value={registerPlaybackRow}>
        {header}
      </RegisterPlaybackRowContext>
      {/* The band clips; what scrolls inside it is the block's own business -
          for the timeline, the lane rows alone, with the ruler and the meter
          fixed around them. The transport row owns its inset, so this takes
          one for the block. */}
      <div
        // The ruler, rows and meter carry the inset; the scroller spans it.
        className="min-h-0 flex-1 overflow-hidden"
        ref={contentRef}
      >
        <ReportBlockHeightContext value={reportBlockHeight}>
          {children}
        </ReportBlockHeightContext>
      </div>
    </div>
  );
}
