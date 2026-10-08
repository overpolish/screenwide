// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// A right-to-left language mirrors the layout, so sides are named by where
// text starts and ends rather than by left and right. This flags physical
// Tailwind classes in any string (`ml-2`, `left-0`, `text-right`,
// `rounded-l-md`, `border-r`) and physical keys in a JSX `style` object
// (`left`, `marginRight`), naming the logical one to use. A class behind an
// `rtl:` or `ltr:` variant is a deliberate side and passes. `translate-x`
// has no logical form: where it moves something sideways, pair it with an
// `rtl:` twin by hand.

const CLASSES = [
  [/^(-?)m([lr])-/u, (sign, side) => `${sign}m${side === "l" ? "s" : "e"}-`],
  [/^p([lr])-/u, (side) => `p${side === "l" ? "s" : "e"}-`],
  [
    /^(-?)(left|right)-/u,
    (sign, side) => `${sign}${side === "left" ? "start" : "end"}-`,
  ],
  [
    /^text-(left|right)$/u,
    (side) => `text-${side === "left" ? "start" : "end"}`,
  ],
  [
    /^float-(left|right)$/u,
    (side) => `float-${side === "left" ? "start" : "end"}`,
  ],
  [
    /^clear-(left|right)$/u,
    (side) => `clear-${side === "left" ? "start" : "end"}`,
  ],
  [/^rounded-([lr])(?=-|$)/u, (side) => `rounded-${side === "l" ? "s" : "e"}`],
  [
    /^rounded-([tb])([lr])(?=-|$)/u,
    (edge, side) =>
      `rounded-${edge === "t" ? "s" : "e"}${side === "l" ? "s" : "e"}`,
  ],
  [/^border-([lr])(?=-|$)/u, (side) => `border-${side === "l" ? "s" : "e"}`],
  [
    /^scroll-([mp])([lr])-/u,
    (box, side) => `scroll-${box}${side === "l" ? "s" : "e"}-`,
  ],
];

const STYLE_KEYS = {
  borderLeft: "borderInlineStart",
  borderRight: "borderInlineEnd",
  left: "insetInlineStart",
  marginLeft: "marginInlineStart",
  marginRight: "marginInlineEnd",
  paddingLeft: "paddingInlineStart",
  paddingRight: "paddingInlineEnd",
  right: "insetInlineEnd",
};

/** The logical form of a physical class, or null when `token` is fine. */
function logicalClass(token) {
  const variants = token.split(":");
  const utility = variants.pop() ?? "";
  if (variants.includes("rtl") || variants.includes("ltr")) return null;
  const bare = utility.replace(/^!/u, "");
  for (const [pattern, logical] of CLASSES) {
    const match = pattern.exec(bare);
    if (match) {
      return [
        ...variants,
        bare.replace(pattern, logical(...match.slice(1))),
      ].join(":");
    }
  }
  return null;
}

const keyName = (key) =>
  key.type === "Identifier"
    ? key.name
    : key.type === "Literal"
      ? key.value
      : null;

/** The object literals a `style` value can be: itself, or either branch. */
function stylesIn(node) {
  if (!node) return [];
  if (node.type === "ObjectExpression") return [node];
  if (node.type === "ConditionalExpression") {
    return [...stylesIn(node.consequent), ...stylesIn(node.alternate)];
  }
  if (node.type === "LogicalExpression") return stylesIn(node.right);
  return [];
}

export const logicalDirection = {
  create(context) {
    const checkText = (node, text) => {
      for (const token of text.split(/\s+/u)) {
        const logical = logicalClass(token);
        if (logical) {
          context.report({
            data: { logical, token },
            messageId: "class",
            node,
          });
        }
      }
    };
    return {
      JSXAttribute(node) {
        if (node.name.name !== "style") return;
        for (const object of stylesIn(node.value?.expression)) {
          for (const property of object.properties) {
            if (property.type !== "Property" || property.computed) continue;
            const name = keyName(property.key);
            const logical =
              typeof name === "string" ? STYLE_KEYS[name] : undefined;
            if (logical) {
              context.report({
                data: { logical, token: name },
                messageId: "style",
                node: property,
              });
            }
          }
        }
      },
      Literal(node) {
        if (typeof node.value === "string") checkText(node, node.value);
      },
      TemplateElement(node) {
        checkText(node, node.value.cooked ?? "");
      },
    };
  },
  meta: {
    docs: {
      description:
        "Name sides by start and end so layouts mirror right to left",
    },
    messages: {
      class:
        '"{{token}}" stays on one side in right-to-left languages; use "{{logical}}", or an rtl:/ltr: variant for a deliberate side.',
      style:
        '"{{token}}" stays on one side in right-to-left languages; use "{{logical}}".',
    },
    schema: [],
    type: "suggestion",
  },
};
