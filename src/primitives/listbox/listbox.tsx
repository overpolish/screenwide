// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Ref } from "react";
import {
  ListBox as AriaListBox,
  ListBoxProps as AriaListBoxProps,
} from "react-aria-components";

import { t } from "../../i18n/i18n";
import { tv } from "../../lib/variants";

// A list lives in a native panel window, which supplies the material, so it
// draws no surface of its own. As in a native menu it pads the panel by the
// tight inset and lets its items touch, so the highlight of neighbouring
// items runs on as one. An empty list says so in a row drawn as a disabled
// item is, its words centred in the list, since there is no column of labels
// for them to line up with.
const listBoxVariants = tv({
  base: "flex w-(--trigger-width) scroll-py-5 flex-col overflow-auto p-inset-sm text-fg outline-none",
});

type ListBoxProps<T extends object> = AriaListBoxProps<T> & {
  className?: string;
  ref?: Ref<HTMLDivElement>;
};

export const ListBox = <T extends object>({
  children,
  className,
  ref,
  ...props
}: ListBoxProps<T>) => {
  return (
    <AriaListBox
      ref={ref}
      renderEmptyState={() => (
        <span className="flex h-control-height items-center justify-center px-inset-sm text-body opacity-40">
          {t("controls-no-items")}
        </span>
      )}
      {...props}
      className={listBoxVariants({ className })}
    >
      {children}
    </AriaListBox>
  );
};
