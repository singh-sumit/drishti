import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'Drishti Docs',
  tagline: 'eBPF observability daemon engineering and operations guide',
  favicon: 'drishti_logo/favicon.ico',
  future: {
    v4: true,
  },
  url: 'https://singh-sumit.github.io',
  baseUrl: '/drishti/',
  organizationName: 'singh-sumit',
  projectName: 'drishti',
  onBrokenLinks: 'throw',
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },
  markdown: {
    mermaid: true,
    hooks: {
      onBrokenMarkdownLinks: 'throw',
    },
  },
  themes: ['@docusaurus/theme-mermaid'],
  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          editUrl: 'https://github.com/singh-sumit/drishti/tree/main/',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],
  themeConfig: {
    image: 'img/docusaurus-social-card.jpg',
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'Drishti Docs',
      logo: {
        alt: 'Drishti',
        src: 'drishti_logo/android-chrome-192x192.png',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Engineering Docs',
        },
        {
          href: 'https://github.com/singh-sumit/drishti',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {
              label: 'Intro',
              to: '/docs/intro',
            },
            {
              label: 'Spec Alignment',
              to: '/docs/engineering/spec-alignment',
            },
          ],
        },
        {
          title: 'Integrations',
          items: [
            {
              label: 'Prometheus',
              to: '/docs/integrations/prometheus',
            },
            {
              label: 'Grafana',
              to: '/docs/integrations/grafana',
            },
            {
              label: 'Systemd',
              to: '/docs/integrations/systemd',
            },
          ],
        },
        {
          title: 'Project',
          items: [
            {
              label: 'Repository',
              href: 'https://github.com/singh-sumit/drishti',
            },
            {
              label: 'Issues',
              href: 'https://github.com/singh-sumit/drishti/issues',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Drishti contributors.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
