import { Command } from 'commander';
import { spawnSync } from "node:child_process";
import path from "node:path";
import { existsSync, readdirSync, rmSync } from "node:fs";

const program = new Command();

program
    .name('gauntlet-run-scenarios')
    .description('Gauntlet Scenario Runner Tool');

program.command('run-scenarios')
    .argument('[plugin]')
    .action(async (plugin) => {
        await runScenarios(plugin)
    });

program.command('run-screenshot-gen')
    .argument('[plugin]')
    .argument('[entrypoint]')
    .action(async (plugin, entrypoint) => {
        await runScreenshotGen(plugin, entrypoint)
    });

await program.parseAsync(process.argv);

async function sleep(ms: number) {
    return new Promise((r) => setTimeout(r, ms));
}

async function runScenarios(expectedPlugin: string | undefined) {
    const projectRoot = path.resolve(process.cwd(), '..', '..');

    const scenarios = path.join(projectRoot, "scenarios");
    const scenariosData = path.join(scenarios, "data");
    const scenariosRun = path.join(scenarios, "run");

    console.log("Building server")
    buildServer(projectRoot)

    console.log("Building scenario plugins")
    buildScenarioPlugins(projectRoot)

    for (const pluginName of readdirSync(scenariosData)) {
        if (expectedPlugin) {
            if (pluginName != expectedPlugin) {
                continue
            }
        }

        console.log("Starting runner")

        const backendProcess = spawnSync('target/debug/gauntlet', {
            stdio: "inherit",
            cwd: projectRoot,
            env: Object.assign(process.env, {
                RUST_LOG: "server=info",
                GAUNTLET_SCENARIO_RUNNER_TYPE: "scenario_runner",
                GAUNTLET_SCENARIOS_DIR: scenarios,
                GAUNTLET_SCENARIO_PLUGIN_NAME: pluginName,
                XDG_DATA_HOME: path.join(scenariosRun, "data"),
                XDG_CONFIG_HOME: path.join(scenariosRun, "config"),
                XDG_CACHE_HOME: path.join(scenariosRun, "cache"),
                XDG_STATE_HOME: path.join(scenariosRun, "state"),
            })
        })

        if (backendProcess.status !== 0) {
            throw new Error(`Unable to run scenario runner, status: ${JSON.stringify(backendProcess)}`);
        }

        if (existsSync(scenariosRun)) {
            rmSync(scenariosRun, { recursive: true })
        }

        await sleep(1000)
    }
}

async function runScreenshotGen(expectedPlugin: string | undefined, expectedEntrypoint: string | undefined) {
    const projectRoot = path.resolve(process.cwd(), '..', '..');
    const scenarios = path.join(projectRoot, "scenarios");
    const scenariosOut = path.join(scenarios, "out");

    buildServer(projectRoot)

    for (const plugin of readdirSync(scenariosOut)) {
        if (expectedPlugin) {
            if (plugin != expectedPlugin) {
                continue
            }
        }

        const pluginDir = path.join(scenariosOut, plugin);

        for (const entrypoint of readdirSync(pluginDir)) {
            if (expectedEntrypoint) {
                if (entrypoint != expectedEntrypoint) {
                    continue
                }
            }

            const entrypointDir = path.join(pluginDir, entrypoint);

            for (const scenario of readdirSync(entrypointDir)) {
                const scenarioFile = path.join(entrypointDir, scenario);

                console.log("Starting screenshot generating runner for scenario: " + scenarioFile)

                const scenarioName = path.parse(scenario).name;
                const entrypointName = path.parse(entrypoint).name;

                let scenarioNameTitle = entrypointName
                    .split("-")
                    .filter(x => x.length > 0)
                    .map(x => (x.charAt(0).toUpperCase() + x.slice(1)))
                    .join(" ");

                const frontendReturn = spawnSync('target/debug/gauntlet', {
                    stdio: "inherit",
                    cwd: projectRoot,
                    env: Object.assign(process.env, {
                        RUST_LOG: "client=info",
                        GAUNTLET_SCENARIO_RUNNER_TYPE: "screenshot_gen",
                        GAUNTLET_SCREENSHOT_GEN_IN: scenarioFile,
                        GAUNTLET_SCREENSHOT_GEN_OUT: path.join(scenarios, "out-screenshot", plugin, entrypoint, scenarioName + ".png"),
                        GAUNTLET_SCREENSHOT_GEN_NAME: scenarioNameTitle,
                    })
                });

                if (frontendReturn.status !== 0) {
                    throw new Error(`Unable to run frontend, status: ${JSON.stringify(frontendReturn)}`);
                }

                console.log("Runner exited")
            }
        }
    }
}

function buildServer(projectRoot: string) {
    const serverBuildResult = spawnSync('cargo', ['build', '--features', 'scenario_runner'], {
        stdio: "inherit",
        cwd: projectRoot,
        env: Object.assign(process.env, {
            RUST_BACKTRACE: "1"
        })
    });

    if (serverBuildResult.status !== 0) {
        throw new Error(`Unable to compile server, status: ${JSON.stringify(serverBuildResult)}`);
    }
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
