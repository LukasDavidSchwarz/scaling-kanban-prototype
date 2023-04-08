import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';
import eslint from 'vite-plugin-eslint';

export default ({ mode }) => {
    process.env = { ...process.env, ...loadEnv(mode, process.cwd()) };

    return defineConfig({
        build: { outDir: 'build' },
        server: {
            host: process.env.VITE_HOST,
            port: parseInt(process.env.VITE_PORT),
            strictPort: true,
        },
        plugins: [react(), eslint()],
    });
};
