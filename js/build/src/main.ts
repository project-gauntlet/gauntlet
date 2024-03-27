import { Command } from 'commander';
import { readFile, writeFile } from "node:fs/promises";
import { simpleGit } from 'simple-git';
import { EOL } from "node:os";
import { Octokit } from 'octokit';
import { execSync } from "node:child_process";
import path from "node:path";

const program = new Command();

program
    .name('gauntlet-build')
    .description('Gauntlet Build Tool')

program.command('publish')
    .action(async () => {
        await doPublish()
    });

program.command('build')
    .action(async () => {
        await doBuild()
    });

await program.parseAsync(process.argv);

function build(projectRoot: string, check: boolean) {
    console.log("Building js...")
    execSync('npm run build', { stdio: "inherit", cwd: projectRoot});

    if (check) {
        console.log("Checking rust...")
        execSync('cargo check', { stdio: "inherit", cwd: projectRoot });
    }

    console.log("Building rust...")
    execSync('cargo build --release', { stdio: "inherit", cwd: projectRoot });
}

async function doBuild() {
    const projectRoot = path.resolve(process.cwd(), '..', '..');
    build(projectRoot, true)
}

async function doPublish() {
    const projectRoot = path.resolve(process.cwd(), '..', '..');
    const git = simpleGit(projectRoot);

    const versionFilePath = path.join(projectRoot, "VERSION");
    const changelogFilePath = path.join(projectRoot, "CHANGELOG.md");
    const denoProjectPath = path.join(projectRoot, "js", "deno");
    const apiProjectPath = path.join(projectRoot, "js", "api");

    console.log("Reading version file...")
    const versionRaw = await readFile(versionFilePath, { encoding: "utf-8" });
    const oldVersion = Number(versionRaw.trim());

    const newVersion = oldVersion + 1;

    console.log("Writing version file...")
    await writeFile(versionFilePath, `${newVersion}`)

    console.log("Reading changelog file...")
    const changelogRaw = await readFile(changelogFilePath, { encoding: "utf-8" });

    const notesForCurrentRelease: string[] = []
    const newChangelog: string[] = []

    let section: "before" | "unreleased" | "after" = "before" ;
    for (const line of changelogRaw.split(EOL)) {
        switch (section) {
            case "before": {
                if (line.match(/^## \[Unreleased]/) != null) {
                    section = "unreleased"

                    const date = new Date();
                    const year = date.getFullYear();
                    const month = `0${date.getMonth() + 1}`.slice(-2);
                    const day = `0${date.getDate()}`.slice(-2);

                    const formattedDate = `${year}-${month}-${day}`;

                    newChangelog.push(line)
                    newChangelog.push("")
                    newChangelog.push(`## [${newVersion}] - ${formattedDate}`)
                } else {
                    newChangelog.push(line)
                }
                break;
            }
            case "unreleased": {
                newChangelog.push(line)
                if (line.match(/^## /) != null) {
                    section = "after"
                } else {
                    notesForCurrentRelease.push(line)
                }
                break;
            }
            case "after": {
                newChangelog.push(line)
                break;
            }
        }
    }

    console.log("Writing changelog file...")
    await writeFile(changelogFilePath, newChangelog.join(EOL))

    const bumpNpmPackage = (packageDir: string) => {
        execSync(`npm version 0.${newVersion}.0`, { stdio: "inherit", cwd: packageDir })
    }

    console.log("Bump version for deno subproject...")
    bumpNpmPackage(denoProjectPath)
    console.log("Bump version for api subproject...")
    bumpNpmPackage(apiProjectPath)

    console.log("git add all files...")
    await git.raw('add', '-A')
    console.log("git commit...")
    await git.commit(`Release v${newVersion}`);
    console.log("git add version tag...")
    await git.addTag(`v${newVersion}`)
    console.log("git push...")
    await git.push()
    console.log("git push tags...")
    await git.pushTags();

    build(projectRoot, false)

    console.log("Publishing npm deno package...")
    execSync('npm publish', { stdio: "inherit", cwd: denoProjectPath })
    console.log("Publishing npm api package...")
    execSync('npm publish', { stdio: "inherit", cwd: apiProjectPath })

    console.log("Fetching architecture and target...")
    const rustcVv = execSync('rustc -Vv', { encoding: "utf-8" });
    console.log(rustcVv)
    const archTarget = rustcVv.match(/^host: (.+)$/m)!![1]

    const octokit = new Octokit({
        auth: process.env.GITHUB_TOKEN
    })

    const repo = { owner: 'project-gauntlet', repo: 'gauntlet' };

    console.log("Creating github release...")
    const response = await octokit.rest.repos.createRelease({
        ...repo,
        tag_name: `v${newVersion}`,
        target_commitish: 'main',
        name: `v${newVersion}`,
        body: notesForCurrentRelease.join('\n'),
    });

    const executableFilePath = path.join(projectRoot, 'target', 'release', 'gauntlet');
    const fileBuffer = await readFile(executableFilePath);

    console.log("Uploading executable as github release asset...")
    await octokit.rest.repos.uploadReleaseAsset({
        ...repo,
        release_id: response.data.id,
        origin: response.data.upload_url,
        name: `gauntlet-${archTarget}`,
        headers: {
            "Content-Type": "application/octet-stream",
        },
        data: fileBuffer as any,
    })
}
