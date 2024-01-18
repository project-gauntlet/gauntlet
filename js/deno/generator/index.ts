import { existsSync, mkdirSync, writeFileSync } from "node:fs";

// https://github.com/denoland/deno/releases/tag/v1.36.4
const LIB_DENO_DECLARATION_URL = "https://github.com/denoland/deno/releases/download/v1.36.4/lib.deno.d.ts";

const res = await fetch(LIB_DENO_DECLARATION_URL);
const content = await res.text();

const fixedContent = content.replaceAll(/\/\/\/ <reference lib="deno\..*" \/>/g, "")

const distDir = "./dist";
if (!existsSync(distDir)) {
    mkdirSync(distDir);
}

writeFileSync(`${distDir}/lib.deno.d.ts`, fixedContent)
