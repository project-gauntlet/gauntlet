import * as esbuild from 'esbuild'

await esbuild.build({
    entryPoints: ['src/main.tsx'],
    bundle: true,
    outfile: 'dist/main.js',
    target: 'deno1.33',
})