import {Command} from 'commander';
import {spawn, spawnSync} from "node:child_process";
import path from "node:path";
import {existsSync, readdirSync, rmSync} from "node:fs";

const program = new Command();

program
    .name('gauntlet-run-scenarios')
    .description('Gauntlet Scenario Runner Tool');

program.command('run-scenarios')
    .action(async () => {
        await runScenarios()
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

    const scenarioRunnerBuildResult = spawnSync('cargo', ['build', '--package', 'scenario_runner'], {
        stdio: "inherit",
        cwd: projectRoot,
    });

    if (scenarioRunnerBuildResult.status !== 0) {
        throw new Error(`Unable to compile generator, status ${scenarioRunnerBuildResult.status}`);
    }

    const serverBuildResult = spawnSync('cargo', ['build', '--features', 'scenario_runner'], {
        stdio: "inherit",
        cwd: projectRoot,
    });

    if (serverBuildResult.status !== 0) {
        throw new Error(`Unable to compile server, status: ${serverBuildResult.status}`);
    }

    for (const scenarioName of readdirSync(scenariosData)) {

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

        spawnSync('target/debug/scenario_runner', ['frontend'], {
            stdio: "inherit",
            cwd: projectRoot,
            env: Object.assign(process.env, {
                RUST_LOG: "info",
                GAUNTLET_SCENARIOS_DIR: scenarios,
                GAUNTLET_SCENARIO_PLUGIN_NAME: scenarioName,
            })
        })

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
