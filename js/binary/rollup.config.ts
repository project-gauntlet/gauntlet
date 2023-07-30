import typescript from '@rollup/plugin-typescript';
import { defineConfig } from "rollup";

export default defineConfig({
    input: [
        'src/main.ts'
    ],
    output: [
        {
            dir: 'dist',
            format: 'esm',
            sourcemap: true
        }
    ],
    plugins: [
        typescript({
            tsconfig: './tsconfig.json',
        }),
    ]
})
