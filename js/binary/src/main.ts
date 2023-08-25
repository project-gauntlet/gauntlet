import nodeResolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import typescript from '@rollup/plugin-typescript';
import { rollup } from "rollup";
import { readFileSync } from "node:fs";

interface PackageJson {
    plugin: PackageJsonPlugin
}
interface PackageJsonPlugin {
    entrypoints: PackageJsonPluginEntrypoint[]
}
interface PackageJsonPluginEntrypoint {
    id: string
    path: string
}


// TODO separate config file, to not tie to package.json and json in general
const text: string = readFileSync("./package.json", "utf8");
const packageJson = JSON.parse(text) as PackageJson;
const mapInputs = packageJson.plugin.entrypoints.map(entrypoint => [entrypoint.id, entrypoint.path] as const);
const entries = new Map(mapInputs);
const inputs = Object.fromEntries(entries);

const rollupBuild = await rollup({
    input: inputs,
    external: ["react", "react/jsx-runtime"],
    plugins: [
        nodeResolve(),
        commonjs(),
        typescript({
            tsconfig: './tsconfig.json',
        }),
    ]
});

await rollupBuild.write({
    dir: 'dist',
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
