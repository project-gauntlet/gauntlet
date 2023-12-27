import nodeResolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import typescript from '@rollup/plugin-typescript';
import { rollup } from "rollup";
import { readFileSync, writeFileSync } from "node:fs";
import { parse as parseToml } from "toml";
import { z } from "zod";

const Config = z.strictObject({
    gauntlet: z.strictObject({
        name: z.string()
    }),
    entrypoint: z.array(z.strictObject({
        id: z.string(),
        name: z.string(),
        path: z.string(),
        type: z.enum(["command", "view"])
    })),
    permissions: z.strictObject({
        environment: z.array(z.string()).default([]),
        high_resolution_time: z.boolean().default(false),
        network: z.array(z.string()).default([]),
        ffi: z.array(z.string()).default([]),
        fs_read_access: z.array(z.string()).default([]),
        fs_write_access: z.array(z.string()).default([]),
        run_subprocess: z.array(z.string()).default([]),
        system: z.array(z.string()).default([]),
    }).default({}),
    supported_system: z.array(z.discriminatedUnion("os", [
        z.strictObject({ os: z.literal("linux") })
    ])).default([]),
});

const text: string = readFileSync("./gauntlet.toml", "utf8");
const config = Config.parse(parseToml(text));

const permEnvExist = config.permissions.environment.length !== 0;
const permFfiExist = config.permissions.ffi.length !== 0;
const permFsReadExist = config.permissions.fs_read_access.length !== 0;
const permFsWriteExist = config.permissions.fs_write_access.length !== 0;
const permRunExist = config.permissions.run_subprocess.length !== 0;
const permSystemExist = config.permissions.system.length !== 0;

if (permEnvExist || permFfiExist || permFsReadExist || permFsWriteExist || permRunExist || permSystemExist) {
    if (config.supported_system.length === 0) {
        throw new Error('Permissions "environment", "ffi", "fs_read_access", "fs_write_access", "run_subprocess", "system" require you to specify "supported_system"')
    }
}

const mapInputs = config.entrypoint.map(entrypoint => [entrypoint.id, entrypoint.path] as const);
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
