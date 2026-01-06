# AGENTS.md - jolt Website

> See root `/AGENTS.md` for repo-wide conventions.

Astro + Starlight documentation site for jolt.

## Structure

```
website/
├── src/
│   ├── assets/          # Images, logo.svg
│   ├── components/      # Custom Astro components
│   ├── content/
│   │   └── docs/        # Markdown documentation pages
│   ├── layouts/         # Page layouts
│   ├── pages/           # Non-docs pages
│   ├── styles/          # Custom CSS (custom.css)
│   └── content.config.ts
├── public/              # Static assets (favicons)
├── astro.config.mjs     # Starlight config, sidebar, social links
└── package.json         # Scripts, prettier config
```

## Commands

Run from `website/` directory:

| Task | Command |
|------|---------|
| Dev server | `bun dev` (localhost:4321) |
| Build | `bun run build` |
| Preview build | `bun preview` |
| Lint | `bun run lint` |
| Lint + fix | `bun run lint:fix` |
| Format check | `bun run format:check` |
| Format | `bun run format` |
| Type check | `bun run check` |

## Code Style

### Prettier (configured in package.json)
- No semicolons
- Single quotes
- 2-space tabs
- Trailing commas
- 100 char line width

### TypeScript
- Unused vars with `_` prefix allowed
- ESLint + @typescript-eslint enforced

## Common Tasks

### Add Documentation Page
1. Create `.md` or `.mdx` file in `src/content/docs/`
2. Add frontmatter with `title` and optional `description`
3. Add to sidebar in `astro.config.mjs`

```md
---
title: Page Title
description: Optional description for SEO
---

Content here...
```

### Modify Sidebar
Edit `sidebar` array in `astro.config.mjs`:
```js
sidebar: [
  {
    label: 'Section Name',
    items: [
      { label: 'Page Title', slug: 'docs/page-slug' },
    ],
  },
]
```

### Add Custom Component
1. Create `.astro` file in `src/components/`
2. Import in markdown: `import Component from '../../components/Component.astro'`
3. Or override Starlight component in `astro.config.mjs` under `components:`

### Add Custom Styles
Edit `src/styles/custom.css` - imported via `customCss` in astro.config.mjs

## Deployment

- Site: `https://jordond.github.io/jolt`
- Base path: `/jolt` (configured in astro.config.mjs)
- Built to `dist/` directory
