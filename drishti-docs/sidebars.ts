import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docsSidebar: [
    'intro',
    'what-it-solves',
    {
      type: 'category',
      label: 'Architecture',
      items: [
        'architecture/overview',
        'architecture/pipeline',
        'architecture/security-and-capabilities',
      ],
    },
    {
      type: 'category',
      label: 'Engineering',
      items: ['engineering/spec-alignment'],
    },
    {
      type: 'category',
      label: 'Usage',
      items: ['usage/quickstart', 'usage/configuration', 'usage/collector-matrix'],
    },
    {
      type: 'category',
      label: 'Integrations',
      items: [
        'integrations/prometheus',
        'integrations/grafana',
        'integrations/systemd',
      ],
    },
    {
      type: 'category',
      label: 'Operations',
      items: ['operations/troubleshooting', 'operations/performance-budgets'],
    },
    {
      type: 'category',
      label: 'Development',
      items: ['development/contributing', 'development/testing-and-ci'],
    },
    {
      type: 'category',
      label: 'Reference',
      items: ['reference/metrics', 'reference/api-reference'],
    },
    'roadmap',
  ],
};

export default sidebars;
