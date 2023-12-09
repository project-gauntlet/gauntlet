import nodeResolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import typescript from '@rollup/plugin-typescript';
import { defineConfig, RollupOptions } from "rollup";

const config = (outDir: string): RollupOptions => {
    return {
        input: [
            'src/init.ts',
        ],
        output: [
            {
                dir: outDir,
                format: 'esm',
                sourcemap: 'inline',
            }
        ],
        plugins: [
            nodeResolve(),
            commonjs(),
            typescript({
                tsconfig: './tsconfig.json',
            }),
        ]
    }
}

export default defineConfig([
    config('dist/prod'),
    config('dist/dev')
])
