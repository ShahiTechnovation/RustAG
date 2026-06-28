// Information architecture for the RustAG docs. Single source of truth for the
// sidebar, the command palette, and prev/next pagination.

export type DocLink = {
  title: string;
  href: string;
  /** Short status chip, e.g. "Preview", "New". */
  badge?: string;
};

export type DocGroup = {
  label: string;
  items: DocLink[];
};

export const DOCS_NAV: DocGroup[] = [
  {
    label: "Get started",
    items: [
      { title: "Introduction", href: "/docs" },
      { title: "Quickstart", href: "/docs/quickstart" },
    ],
  },
  {
    label: "Core concepts",
    items: [
      { title: "The lazy mirror", href: "/docs/concepts" },
      { title: "Account state machine", href: "/docs/concepts#state-machine" },
      { title: "Oracle freshness", href: "/docs/concepts#oracles" },
    ],
  },
  {
    label: "Reference",
    items: [
      { title: "CLI", href: "/docs/cli" },
      { title: "TypeScript SDK", href: "/docs/sdk" },
      { title: "REST API", href: "/docs/sdk#rest-api" },
      { title: "RPC compatibility", href: "/docs/sdk#rpc" },
    ],
  },
  {
    label: "Architecture",
    items: [
      { title: "System overview", href: "/docs/architecture" },
      { title: "Crate map", href: "/docs/architecture#crates" },
      { title: "Phase 2 & 3", href: "/docs/architecture#phases", badge: "Preview" },
    ],
  },
  {
    label: "Trust & operations",
    items: [
      { title: "Security & attestation", href: "/docs/security" },
      { title: "Known limitations", href: "/docs/security#limitations" },
      { title: "FAQ", href: "/docs/security#faq" },
    ],
  },
];

/** Canonical, ordered list of real pages (one per route) for prev/next paging. */
export const DOC_ORDER: DocLink[] = [
  { title: "Introduction", href: "/docs" },
  { title: "Quickstart", href: "/docs/quickstart" },
  { title: "Core concepts", href: "/docs/concepts" },
  { title: "CLI reference", href: "/docs/cli" },
  { title: "SDK & API", href: "/docs/sdk" },
  { title: "Architecture", href: "/docs/architecture" },
  { title: "Trust & security", href: "/docs/security" },
];

/** Flattened, de-duped link list for the command palette. */
export const DOCS_SEARCH_INDEX: (DocLink & { group: string })[] = DOCS_NAV.flatMap((g) =>
  g.items.map((i) => ({ ...i, group: g.label })),
);
