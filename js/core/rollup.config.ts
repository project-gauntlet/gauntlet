import nodeResolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import typescript from '@rollup/plugin-typescript';
import { defineConfig } from "rollup";

export default defineConfig({
    input: [
        'src/init.tsx',
    ],
    output: [
        {
            dir: 'dist',
            format: 'esm',
            sourcemap: 'inline',
        }
    ],
    external: ["react", "react/jsx-runtime"],
    plugins: [
        nodeResolve(),
        commonjs(),
        typescript({
            tsconfig: './tsconfig.json',
        }),
    ]
})
