import commonjs from '@rollup/plugin-commonjs';
import replace from "@rollup/plugin-replace";
import { defineConfig, RollupOptions } from "rollup";

const config = (nodeEnv: string, reactBundle: string, sourceMap: boolean, outDir: string): RollupOptions => {
    return {
        input: [
            `node_modules/react/cjs/react.${reactBundle}`,
            `node_modules/react/cjs/react-jsx-runtime.${reactBundle}`,
        ],

        output: {
            dir: outDir,
            format: 'esm',
        },
        external: ['react'],
        plugins: [
            commonjs(),
            replace({
                'process.env.NODE_ENV': JSON.stringify(nodeEnv)
            })
        ]
    }
}

// 'node_modules/react/cjs/react.development.js',
// 'node_modules/react/cjs/react.production.min.js',
// 'node_modules/react/cjs/react-jsx-dev-runtime.development.js,' # essentially same as react-jsx-runtime.development.js but jsx -> jsxDev, and no jsxs
// 'node_modules/react/cjs/react-jsx-dev-runtime.production.min.js', # empty
// 'node_modules/react/cjs/react-jsx-dev-runtime.profiling.min.js', # empty
// 'node_modules/react/cjs/react-jsx-runtime.development.js',
// 'node_modules/react/cjs/react-jsx-runtime.production.min.js',
// 'node_modules/react/cjs/react-jsx-runtime.profiling.min.js', # same as react-jsx-runtime.production.min.js

export default defineConfig([
    config('production', 'production.min.js', false, 'dist/prod'),
    config('development', 'development.js', true, 'dist/dev')
])
