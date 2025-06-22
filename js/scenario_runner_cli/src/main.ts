import { Command } from 'commander';
import { spawnSync } from "node:child_process";
import path from "node:path";
import { existsSync, rmSync } from "node:fs";

const program = new Command();

program
    .name('gauntlet-run-scenarios')
    .description('Gauntlet Scenario Runner Tool');

program.command('run-scenarios')
    .argument('[plugin]')
    .argument('[entrypoint]')
    .action(async (plugin, entrypoint) => {
        await runScenarios(plugin, entrypoint)
    });

await program.parseAsync(process.argv);

async function runScenarios(
    expectedPlugin: string | undefined,
    expectedEntrypoint: string | undefined
) {
    const projectRoot = path.resolve(process.cwd(), '..', '..');

    console.log("Building scenario plugins...")
    buildScenarioPlugins(projectRoot)

    console.log("Running scenario runner...")
    const scenarios = path.join(projectRoot, "example_plugins");
    const scenariosRun = path.join(scenarios, "run");

    const env: Record<string, string> = {
        RUST_BACKTRACE: "1",
        RUST_LOG: "gauntlet_server=INFO,gauntlet_client=INFO,gauntlet_scenario_runner=INFO",
        XDG_DATA_HOME: path.join(scenariosRun, "data"),
        XDG_CONFIG_HOME: path.join(scenariosRun, "config"),
        XDG_CACHE_HOME: path.join(scenariosRun, "cache"),
        XDG_STATE_HOME: path.join(scenariosRun, "state"),
        GAUNTLET_SCENARIOS_DIR: path.join(scenarios, "scenarios"),
        GAUNTLET_SCENARIOS_PLUGINS_DIR: path.join(scenarios, "plugins"),
        GAUNTLET_SCENARIOS_SCREENSHOTS_DIR: path.join(scenarios, "out_screenshot"),
    };

    if (expectedPlugin) {
        env.GAUNTLET_SCENARIOS_ONLY_PLUGIN = expectedPlugin;
    }
    if (expectedEntrypoint) {
        env.GAUNTLET_SCENARIOS_ONLY_ENTRYPOINT = expectedEntrypoint;
    }

    const runReturn = spawnSync('cargo', ['run', '--package', 'gauntlet-scenario-runner', '--features', 'scenario_runner'], {
        stdio: "inherit",
        cwd: projectRoot,
        env: Object.assign(process.env, env)
    });

    if (runReturn.status !== 0) {
        throw new Error(`Unable to run scenario, status: ${JSON.stringify(runReturn)}`);
    }

    console.log("Runner is done, cleaning up...")

    if (existsSync(scenariosRun)) {
        rmSync(scenariosRun, { recursive: true })
    }

    console.log("Done")
}

function buildScenarioPlugins(projectRoot: string) {
    const scenarioPluginBuildResult = spawnSync('npm', ['run', 'build-all'], {
        stdio: "inherit",
        cwd: projectRoot,
    });

    if (scenarioPluginBuildResult.status !== 0) {
        throw new Error(`Unable to compile plugin, status ${JSON.stringify(scenarioPluginBuildResult)}`);
    }
}
