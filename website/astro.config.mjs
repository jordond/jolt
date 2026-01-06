// @ts-check
import { defineConfig } from 'astro/config'
import starlight from '@astrojs/starlight'

// https://astro.build/config
export default defineConfig({
  site: 'https://jordond.github.io',
  base: '/jolt',
  integrations: [
    starlight({
      title: 'jolt',
      logo: {
        src: './src/assets/logo.svg',
      },
      social: [
        {
          icon: 'github',
          label: 'GitHub',
          href: 'https://github.com/jordond/jolt',
        },
      ],
      customCss: [
        '@fontsource/inter/400.css',
        '@fontsource/inter/600.css',
        '@fontsource/jetbrains-mono/400.css',
        './src/styles/custom.css',
      ],
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { label: 'Introduction', slug: 'docs/getting-started' },
            { label: 'Installation', slug: 'docs/installation' },
            { label: 'Quick Start', slug: 'docs/quick-start' },
          ],
        },
        {
          label: 'Using jolt',
          items: [
            { label: 'TUI Interface', slug: 'docs/tui-interface' },
            { label: 'Keyboard Shortcuts', slug: 'docs/keyboard-shortcuts' },
            { label: 'Understanding Metrics', slug: 'docs/understanding-metrics' },
          ],
        },
        {
          label: 'Configuration',
          items: [
            { label: 'Config File', slug: 'docs/configuration' },
            { label: 'Themes', slug: 'docs/themes' },
            { label: 'Custom Themes', slug: 'docs/custom-themes' },
          ],
        },
        {
          label: 'Advanced',
          items: [
            { label: 'Background Daemon', slug: 'docs/daemon' },
            { label: 'Historical Data', slug: 'docs/historical-data' },
            { label: 'JSON Output', slug: 'docs/json-output' },
          ],
        },
        {
          label: 'Reference',
          items: [
            { label: 'CLI Commands', slug: 'docs/cli-reference' },
            { label: 'FAQ', slug: 'docs/faq' },
            { label: 'Troubleshooting', slug: 'docs/troubleshooting' },
          ],
        },
      ],
      components: {
        Header: './src/components/Header.astro',
      },
    }),
  ],
})
