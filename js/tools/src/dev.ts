import { parseManifest, readManifest, rollupInputOptions, rollupOutputOptions } from "./config";
import { RollupError, watch } from "rollup";
import chalk from "chalk";
import { sessionBus } from "dbus-ts";


export type Interfaces = {
    "dev.projectgauntlet.Server.Management": dev.projectgauntlet.Server.Management
}
export namespace dev {
    export namespace projectgauntlet {
        export namespace Server {
            export interface Management {
                SaveLocalPlugin(plugin_id: string): Promise<void>
            }
        }
    }
}

export async function dev() {
    console.log(chalk.cyanBright(`\nwatching for file changes...`));

    const watcher = watch({
        watch: {
            exclude: [
                '**/node_modules/**',
                '**/.git/**',
                '**/dist/**',
            ],
        },
        ...rollupInputOptions(parseManifest(readManifest())),
        output: rollupOutputOptions()
    });

    const messageBus = await sessionBus<Interfaces>();

    const management = await messageBus.getInterface(
        "dev.projectgauntlet.Gauntlet",
        "/dev/projectgauntlet/Server",
        "dev.projectgauntlet.Server.Management"
    );

    watcher.on('event', async (event) => {
        switch (event.code) {
            case "START": {
                break;
            }
            case "BUNDLE_START": {
                console.log(chalk.cyanBright(`\nrefresh started...`));
                break;
            }
            case "BUNDLE_END": {
                const manifestText = readManifest();
                parseManifest(manifestText); // TODO properly handle errors here

                await event.result.close()

                try {
                    await management.SaveLocalPlugin(process.cwd() + "/dist")
                } catch (e) {
                    console.error("Error returned by server");
                    console.error(e);
                }

                console.log(chalk.cyanBright(`refreshed in ${event.duration}ms.`));
                break;
            }
            case "END": {
                break;
            }
            case "ERROR": {
                outputBuildError(event.error)
                break;
            }
        }
    });

    // watcher.close();
}

function outputBuildError(e: RollupError) {
    console.error(chalk.red(`${e.plugin ? `[${e.plugin}] ` : ''}${e.message}`))
    if (e.id) {
        const loc = e.loc ? `:${e.loc.line}:${e.loc.column}` : ''
        console.error(`file: ${chalk.cyan(`${e.id}${loc}`)}`)
    }
    if (e.frame) {
        console.error(chalk.yellow(e.frame))
    }
}