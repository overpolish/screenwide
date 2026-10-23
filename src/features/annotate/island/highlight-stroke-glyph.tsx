// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** How a highlight is drawn, as its band alone in the text's colour, as the
 * tool's icon is. Clean, a straight band; by hand, the renderer's marker
 * stroke (`highlight_stroke` in `annotation_highlight.wgsl`): its top and
 * bottom waving gently and out of step, and both ends cut on the one slant
 * of the nib. The wave is deeper than the renderer's so it reads at this
 * size. Both bands are drawn inset and stroked round, which rounds their
 * corners as the renderer's are. */
export function StrokeGlyph({ isHandDrawn }: { isHandDrawn: boolean }) {
  return (
    <svg
      aria-hidden
      className="size-icon-lg"
      fill="currentColor"
      stroke="currentColor"
      strokeLinejoin="round"
      strokeWidth={2.5}
      viewBox="0 0 24 24"
    >
      {isHandDrawn ? (
        <path d="M5.75 9.25C9 8.25 11.5 10.25 14.75 9.25S18.75 8.5 20.5 9.25L18.25 14.75C15.25 15.75 12.5 13.75 9.25 14.75S5.25 15.5 3.5 14.75Z" />
      ) : (
        <rect height={5.5} width={15.5} x={4.25} y={9.25} />
      )}
    </svg>
  );
}
