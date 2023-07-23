import nodeResolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import typescript from '@rollup/plugin-typescript';
import replace from "@rollup/plugin-replace";
import { defineConfig, RollupOptions } from "rollup";

const config = (nodeEnv: string, sourceMap: boolean, outDir: string): RollupOptions => {
    return {
        input: [
            'src/renderer.tsx',
        ],
        output: [
            {
                dir: outDir,
                format: 'esm',
                sourcemap: sourceMap,
            }
        ],
        external: ["react", "react/jsx-runtime"],
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
            })
        ]
    }
}

export default defineConfig([
    config("production", false, 'dist/prod'),
    config("development",  true, 'dist/dev')
])
