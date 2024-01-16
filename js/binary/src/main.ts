import { Command } from 'commander';
import { build } from "./build";
import { dev } from "./dev";
import { version } from "../package.json";

if (version === undefined) {
    throw new Error("Unexpected error. Version is not available")
}

const program = new Command();

program
    .name('gauntlet')
    .description('Gauntlet CLI')
    .version(version, '-v, --version')

program.command('dev')
    .description('Run a dev server that automatically refreshes plugin when source code changes detected')
    .action(async () => {
        await dev()
    });

program.command('build')
    .description('Build a plugin')
    .action(async () => {
        await build()
    });

await program.parseAsync(process.argv);
