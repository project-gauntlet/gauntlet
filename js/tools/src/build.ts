import { rollup, RollupBuild } from "rollup";
import { parseManifest, readManifest, rollupInputOptions, rollupOutputOptions, writeDistManifest } from "./config";


export async function build() {
    const manifestText = readManifest();

    const manifest = parseManifest(manifestText);

    let rollupBuild: RollupBuild | undefined;
    let buildFailed = false;
    try {
        rollupBuild = await rollup(rollupInputOptions(manifest));

        await rollupBuild.write(rollupOutputOptions());

        writeDistManifest(manifestText)
    } catch (error) {
        buildFailed = true;
        console.error(error);
    }
    
    if (rollupBuild) {
        await rollupBuild.close();
    }

    process.exit(buildFailed ? 1 : 0);
}

