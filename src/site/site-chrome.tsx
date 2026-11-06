// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import mark from "../assets/screenwide-mark.svg";
import { t } from "../i18n/i18n";
import { cn } from "../lib/styling";
import { Text } from "../primitives/text/text";

const RELEASES = "https://github.com/overpolish/screenwide/releases/latest";

/** An SVG drawn in the text's colour, so it follows the appearance. */
export function MaskIcon({
  className,
  src,
}: {
  className: string;
  src: string;
}) {
  return (
    <span
      aria-hidden
      className={cn("shrink-0 bg-current", className)}
      style={{
        // Vite inlines small files as data URLs, which must be quoted in CSS.
        mask: `url(${JSON.stringify(src)}) center / contain no-repeat`,
      }}
    />
  );
}

/** The mark and the name, leading home from any page but the home page. */
export function SiteBrand({
  className,
  isHome = false,
}: {
  className?: string;
  isHome?: boolean;
}) {
  const content = (
    <>
      <MaskIcon className="size-icon-lg" src={mark} />
      {t("site-name")}
    </>
  );
  const style = cn(
    "flex h-control-height items-center gap-inline text-tile-title",
    className,
  );
  return isHome ? (
    <span className={style}>{content}</span>
  ) : (
    <a className={style} href="/">
      {content}
    </a>
  );
}

export function SiteFooter() {
  return (
    <footer className="mt-16 flex flex-wrap justify-between gap-x-group gap-y-inline py-8">
      <Text variant="caption">
        {t("site-footer-licence")}{" "}
        <a className="underline hover:text-fg" href="/privacy">
          {t("site-footer-privacy")}
        </a>
      </Text>
      <Text variant="caption">
        {t("site-footer-requirements")}{" "}
        <a className="underline hover:text-fg" href={RELEASES}>
          {t("site-footer-releases")}
        </a>
      </Text>
    </footer>
  );
}
