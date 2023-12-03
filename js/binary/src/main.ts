import nodeResolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import typescript from '@rollup/plugin-typescript';
import { rollup } from "rollup";
import { readFileSync, writeFileSync } from "node:fs";
import { parse as parseToml } from "toml";

interface Config {
    metadata: ConfigMetadata
    entrypoints: ConfigPluginEntrypoint[]
}

interface ConfigMetadata {
    name: string
}

interface ConfigPluginEntrypoint {
    id: string
    path: string
}

const text: string = readFileSync("./gauntlet.toml", "utf8");
const config = parseToml(text) as Config;
const mapInputs = config.entrypoints.map(entrypoint => [entrypoint.id, entrypoint.path] as const);
const entries = new Map(mapInputs);
const inputs = Object.fromEntries(entries);

const rollupBuild = await rollup({
    input: inputs,
    external: ["react", "react/jsx-runtime", "@project-gauntlet/api/components"],
    plugins: [
        nodeResolve(),
        commonjs(),
        typescript({
            tsconfig: './tsconfig.json',
        }),
    ]
});

await rollupBuild.write({
    dir: 'dist/js',
    format: 'esm',
    sourcemap: 'inline',
    manualChunks: (id, _meta) => {
        if (id.includes('node_modules') || id === '\x00commonjsHelpers.js') {
            return 'vendor';
        } else {
            return 'shared';
        }
    },
    chunkFileNames: '[name].js'
});

writeFileSync("dist/gauntlet.toml", text)
