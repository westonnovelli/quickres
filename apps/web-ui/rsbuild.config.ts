import { defineConfig } from '@rsbuild/core';
import { pluginReact } from '@rsbuild/plugin-react';

export default defineConfig({
  plugins: [
    // React plugin provides fast refresh & TSX support
    pluginReact(),
  ],
  server: {
    port: 5173,
    open: true,
  },
  output: {
    publicPath: '/',
  },
});
