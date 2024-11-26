import { defineConfig } from "rollup";
import alias from '@rollup/plugin-alias';
import { globSync } from "glob";

export default defineConfig({
    input: globSync('dist/**/*', { nodir: true }),
    output: [
        {
            dir: 'dist',
            format: 'esm',
            sourcemap: 'inline',
            preserveModules: true,
        }
    ],
    external: [/^ext:.+/],
    plugins: [
        alias({
            entries: [
                { find: 'react/jsx-runtime', replacement: 'ext:gauntlet/react-jsx-runtime.js' },
                { find: 'react', replacement: 'ext:gauntlet/react.js' },
            ]
        }),
    ]
})
