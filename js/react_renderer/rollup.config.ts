import nodeResolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import typescript from '@rollup/plugin-typescript';
import replace from "@rollup/plugin-replace";
import { defineConfig, RollupOptions } from "rollup";
import alias from '@rollup/plugin-alias';

const config = (nodeEnv: string, outDir: string): RollupOptions => {
    return {
        input: [
            'src/renderer.ts',
        ],
        output: [
            {
                dir: outDir,
                format: 'esm',
                sourcemap: 'inline',
            }
        ],
        external: [/^ext:.+/],
        plugins: [
            nodeResolve(),
            commonjs({
                esmExternals: (id: string) => id === "react"
            }),
            typescript({
                tsconfig: './tsconfig.json',
            }),
            replace({
                delimiters: ['', ''],
                values: {
                    // npm bundle of React has references to npm process
                    'process.env.NODE_ENV': JSON.stringify(nodeEnv),
                    // To make react 7-bit ascii compatible
                    '–': "-",
                    '—': "-"
                }
            }),
            alias({
                entries: [
                    { find: 'react/jsx-runtime', replacement: 'ext:gauntlet/react-jsx-runtime.js' },
                    { find: 'react', replacement: 'ext:gauntlet/react.js' },
                ]
            }),
        ]
    }
}

export default defineConfig([
    config("production", 'dist/prod'),
    config("development", 'dist/dev')
])
