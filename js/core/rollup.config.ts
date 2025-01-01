import nodeResolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import typescript from '@rollup/plugin-typescript';
import { defineConfig } from "rollup";
import alias from '@rollup/plugin-alias';

export default defineConfig({
    input: [
        'src/core.tsx',
        'src/init.ts',
        'src/internal-all.ts',
        'src/internal-linux.ts',
        'src/internal-macos.ts',
        'src/internal-windows.ts',
    ],
    output: [
        {
            dir: 'dist',
            format: 'esm',
            sourcemap: 'inline',
        }
    ],
    external: [/^ext:.+/],
    plugins: [
        nodeResolve(),
        commonjs(),
        typescript({
            tsconfig: './tsconfig.json',
        }),
        alias({
            entries: [
                { find: 'react/jsx-runtime', replacement: 'ext:gauntlet/react-jsx-runtime.js' },
                { find: 'react', replacement: 'ext:gauntlet/react.js' },
            ]
        }),
    ]
})
