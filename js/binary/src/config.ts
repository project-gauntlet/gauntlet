import { z } from "zod";
import { parse as parseToml } from "toml";
import { InputOptions, OutputOptions } from "rollup";
import nodeResolve from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import typescript from "@rollup/plugin-typescript";
import { readFileSync, writeFileSync } from "node:fs";

const Manifest = z.strictObject({
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

type Manifest = z.infer<typeof Manifest>;

export function readManifest(): string {
    return readFileSync("./gauntlet.toml", "utf8")
}

export function writeDistManifest(manifestText: string) {
    writeFileSync("dist/gauntlet.toml", manifestText)
}

export function parseManifest(manifestText: string) {
    const manifest = Manifest.parse(parseToml(manifestText));

    const permEnvExist = manifest.permissions.environment.length !== 0;
    const permFfiExist = manifest.permissions.ffi.length !== 0;
    const permFsReadExist = manifest.permissions.fs_read_access.length !== 0;
    const permFsWriteExist = manifest.permissions.fs_write_access.length !== 0;
    const permRunExist = manifest.permissions.run_subprocess.length !== 0;
    const permSystemExist = manifest.permissions.system.length !== 0;

    if (permEnvExist || permFfiExist || permFsReadExist || permFsWriteExist || permRunExist || permSystemExist) {
        if (manifest.supported_system.length === 0) {
            throw new Error('Permissions "environment", "ffi", "fs_read_access", "fs_write_access", "run_subprocess", "system" require you to specify "supported_system"')
        }
    }

    return manifest
}


export function rollupInputOptions(manifest: Manifest): InputOptions {
    const mapInputs = manifest.entrypoint.map(entrypoint => [entrypoint.id, entrypoint.path] as const);
    const entries = new Map(mapInputs);
    const inputs = Object.fromEntries(entries);

    return {
        input: inputs,
        external: ["react", "react/jsx-runtime", "@project-gauntlet/api/components"],
        plugins: [
            nodeResolve(),
            commonjs(),
            typescript({
                tsconfig: './tsconfig.json',
            }),
        ],
    }
}

export function rollupOutputOptions(): OutputOptions {
    return {
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
    }
}
