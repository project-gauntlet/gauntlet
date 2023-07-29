import nodeResolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import typescript from '@rollup/plugin-typescript';
import { defineConfig } from "rollup";

export default defineConfig({
    input: [
        'src/view.tsx',
    ],
    output: [
        {
            dir: 'dist',
            format: 'esm',
            sourcemap: true
        }
    ],
    external: ["react"],
    plugins: [
        nodeResolve(),
        commonjs(),
        typescript({
            tsconfig: './tsconfig.json',
        }),
    ]
})
