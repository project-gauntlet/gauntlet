import {Command} from 'commander';
import {spawn, spawnSync} from "node:child_process";
import path from "node:path";
import {existsSync, readFileSync, readdirSync, rmSync} from "node:fs";

const program = new Command();

program
    .name('gauntlet-run-scenarios')
    .description('Gauntlet Scenario Runner Tool');

program.command('run-scenarios')
    .action(async () => {
        await runScenarios()
    });

program.command('run-screenshot-gen')
    .action(async () => {
        await runScreenshotGen()
    });

await program.parseAsync(process.argv);

async function sleep(ms: number) {
    return new Promise((r) => setTimeout(r, ms));
}

async function runScenarios() {
    const projectRoot = path.resolve(process.cwd(), '..', '..');

    const scenarios = path.join(projectRoot, "scenarios");
    const scenariosData = path.join(scenarios, "data");
    const scenariosRun = path.join(scenarios, "run");

    console.log("Building scenario runner")
    buildScenarioRunner(projectRoot)

    console.log("Building server")
    buildServer(projectRoot)

    for (const scenarioName of readdirSync(scenariosData)) {
        const scenariosPlugin = path.join(scenarios, "plugins", scenarioName);

        console.log("Building scenario plugin")

        buildScenarioPlugin(scenariosPlugin)

        console.log("Starting real server")

        const backendProcess = spawn('target/debug/gauntlet', ['server'], {
            stdio: "inherit",
            cwd: projectRoot,
            env: Object.assign(process.env, {
                RUST_LOG: "server=info",
                XDG_DATA_HOME: path.join(scenariosRun, "data"),
                XDG_CONFIG_HOME: path.join(scenariosRun, "config"),
                XDG_CACHE_HOME: path.join(scenariosRun, "cache"),
            })
        })

        await sleep(1000)

        console.log("Starting mock frontend")

        const runnerReturn = spawnSync('target/debug/scenario_runner', ['frontend'], {
            stdio: "inherit",
            cwd: projectRoot,
            env: Object.assign(process.env, {
                RUST_LOG: "info",
                GAUNTLET_SCENARIOS_DIR: scenarios,
                GAUNTLET_SCENARIO_PLUGIN_NAME: scenarioName,
            })
        });

        if (runnerReturn.status !== 0) {
            throw new Error(`Unable to run scenario runner, status: ${runnerReturn.status}`);
        }

        await sleep(1000)

        console.log("Killing backend")

        if (!backendProcess.kill()) {
            console.error("Unable to kill backend after frontend finished")
        }
    }

    if (existsSync(scenariosRun)) {
        rmSync(scenariosRun, { recursive: true })
    }
}

async function runScreenshotGen() {
    const projectRoot = path.resolve(process.cwd(), '..', '..');
    const scenarios = path.join(projectRoot, "scenarios");
    const scenariosOut = path.join(scenarios, "out");

    buildScenarioRunner(projectRoot)
    buildServer(projectRoot)

    for (const plugin of readdirSync(scenariosOut)) {
        const pluginDir = path.join(scenariosOut, plugin);

        for (const entrypoint of readdirSync(pluginDir)) {
            const entrypointDir = path.join(pluginDir, entrypoint);

            for (const scenario of readdirSync(entrypointDir)) {
                const scenarioFile = path.join(entrypointDir, scenario);

                console.log("Starting screenshot generating frontend for scenario: " + scenarioFile)

                const mockBackendProcess = spawn('target/debug/scenario_runner', ['server'], {
                    stdio: "inherit",
                    cwd: projectRoot,
                })

                await sleep(500)

                const scenarioName = path.parse(scenario).name;
                const entrypointName = path.parse(entrypoint).name;

                let scenarioNameTitle = entrypointName
                    .split("_")
                    .filter(x => x.length > 0)
                    .map(x => (x.charAt(0).toUpperCase() + x.slice(1)))
                    .join(" ");

                const frontendReturn = spawnSync('target/debug/gauntlet', ['server'], {
                    stdio: "inherit",
                    cwd: projectRoot,
                    env: Object.assign(process.env, {
                        RUST_LOG: "client=info",
                        GAUNTLET_INTERNAL_FRONTEND: "",
                        GAUNTLET_SCREENSHOT_GEN_IN: scenarioFile,
                        GAUNTLET_SCREENSHOT_GEN_OUT: path.join(scenarios, "out-screenshot", plugin, entrypoint, scenarioName + ".png"),
                        GAUNTLET_SCREENSHOT_GEN_NAME: scenarioNameTitle,
                    })
                });

                if (frontendReturn.status !== 0) {
                    throw new Error(`Unable to run frontend, status: ${frontendReturn.error}`);
                }

                console.log("Frontend exited")

                console.log("Killing backend")

                if (!mockBackendProcess.kill()) {
                    console.error("Unable to kill backend after frontend finished")
                }
            }
        }
    }
}

function buildServer(projectRoot: string) {
    const serverBuildResult = spawnSync('cargo', ['build', '--features', 'scenario_runner'], {
        stdio: "inherit",
        cwd: projectRoot,
    });

    if (serverBuildResult.status !== 0) {
        throw new Error(`Unable to compile server, status: ${serverBuildResult.status}`);
    }
}

function buildScenarioRunner(projectRoot: string) {
    const scenarioRunnerBuildResult = spawnSync('cargo', ['build', '--package', 'scenario_runner'], {
        stdio: "inherit",
        cwd: projectRoot,
    });

    if (scenarioRunnerBuildResult.status !== 0) {
        throw new Error(`Unable to compile generator, status ${scenarioRunnerBuildResult.status}`);
    }
}

function buildScenarioPlugin(pluginDir: string) {
    const scenarioPluginBuildResult = spawnSync('npm', ['run', 'build'], {
        stdio: "inherit",
        cwd: pluginDir,
    });

    if (scenarioPluginBuildResult.status !== 0) {
        throw new Error(`Unable to compile plugin, status ${scenarioPluginBuildResult.status}`);
    }
}
