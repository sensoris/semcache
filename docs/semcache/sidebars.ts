import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */
const sidebars: SidebarsConfig = {
  tutorialSidebar: [
    'intro',
    'getting-started',
    {
      type: 'category',
      label: 'Installation & Deployment',
      items: [
        'installation/docker',
      ],
    },
    {
      type: 'category',
      label: 'API Reference',
      items: [
        'api/chat-completions',
        'api/headers',
        'api/supported-providers',
      ],
    },
    {
      type: 'category',
      label: 'Configuration',
      items: [
        'configuration/cache-settings',
        'configuration/embedding-model',
        'configuration/monitoring',
      ],
    },
    {
      type: 'category',
      label: 'Monitoring',
      items: [
        'monitoring/metrics',
        'monitoring/admin-dashboard',
      ],
    },
  ],
};

export default sidebars;
