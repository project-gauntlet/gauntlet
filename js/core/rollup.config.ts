import nodeResolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import typescript from '@rollup/plugin-typescript';
import { defineConfig, RollupOptions } from "rollup";

const config = (nodeEnv: string, sourceMap: boolean, outDir: string): RollupOptions => {
    return {
        input: [
            'src/init.ts',
        ],
        output: [
            {
                dir: outDir,
                format: 'esm',
                sourcemap: sourceMap,
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
    config("production", false, 'dist/prod'),
    config("development",  true, 'dist/dev')
])
