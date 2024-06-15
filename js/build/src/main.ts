import { Command } from 'commander';
import { readFile, writeFile } from "node:fs/promises";
import { simpleGit } from 'simple-git';
import { EOL } from "node:os";
import { Octokit } from 'octokit';
import { spawnSync } from "node:child_process";
import path from "node:path";
import { mkdirSync, readFileSync } from "fs";
import { copyFileSync, writeFileSync } from "node:fs";

const program = new Command();

program
    .name('gauntlet-build')
    .description('Gauntlet Build Tool')

program.command('publish-init')
    .action(async () => {
        await doPublishInit()
    });

program.command('publish-linux')
    .action(async () => {
        await doPublishLinux()
    });

program.command('publish-macos')
    .action(async () => {
        await doPublishMacOS()
    });

program.command('publish-final')
    .action(async () => {
        await doPublishFinal()
    });

program.command('build-linux')
    .action(async () => {
        await doBuildLinux()
    });

program.command('build-macos')
    .action(async () => {
        await doBuildMacOS()
    });

await program.parseAsync(process.argv);

async function doBuild(arch: string) {
    console.log("Building Gauntlet...")

    const projectRoot = getProjectRoot();
    build(projectRoot, true, arch)
}

async function doPublishInit() {
    console.log("Publishing Gauntlet... Initiating...")

    const projectRoot = getProjectRoot()

    const { newVersion, releaseNotes } = await makeRepoChanges(projectRoot);

    await createRelease(newVersion, releaseNotes)
}

async function doPublishLinux() {
    console.log("Publishing Gauntlet... Linux...")

    const projectRoot = getProjectRoot()

    const arch = 'x86_64-unknown-linux-gnu';

    build(projectRoot, false, arch)

    const { fileName, filePath } = packageForLinux(projectRoot, arch)

    await addFileToRelease(projectRoot, filePath, fileName)
}

async function doBuildLinux() {
    await doBuild('x86_64-unknown-linux-gnu')
}

async function doPublishMacOS() {
    const projectRoot = getProjectRoot();

    const arch = 'aarch64-apple-darwin';

    build(projectRoot, false, arch)

    const { fileName, filePath } = await packageForMacos(projectRoot, arch)

    await addFileToRelease(projectRoot, filePath, fileName)
}

async function doBuildMacOS() {
    await doBuild('aarch64-apple-darwin')
}

async function undraftRelease(projectRoot: string) {
    const octokit = getOctokit();

    const version = await readVersion(projectRoot)

    const response = await octokit.rest.repos.getReleaseByTag({
        ...getGithubRepo(),
        tag: `v${version}`,
    });

    await octokit.rest.repos.updateRelease({
        ...getGithubRepo(),
        release_id: response.data.id,
        origin: response.data.upload_url,
        draft: false
    });
}

async function doPublishFinal() {
    console.log("Publishing Gauntlet... Finishing up...")
    const projectRoot = getProjectRoot()

    buildJs(projectRoot)

    publishNpmPackage(projectRoot)

    await undraftRelease(projectRoot)
}

function build(projectRoot: string, check: boolean, arch: string) {
    buildJs(projectRoot)

    if (check) {
        console.log("Checking rust...")
        const cargoCheckResult = spawnSync('cargo', ['check', '--features', 'release', '--target', arch], {
            stdio: "inherit",
            cwd: projectRoot
        });

        if (cargoCheckResult.status !== 0) {
            throw new Error(`Unable to check, status: ${JSON.stringify(cargoCheckResult)}`);
        }
    }

    console.log("Building rust...")
    const cargoBuildResult = spawnSync('cargo', ['build', '--release', '--features', 'release', '--target', arch], {
        stdio: "inherit",
        cwd: projectRoot
    });

    if (cargoBuildResult.status !== 0) {
        throw new Error(`Unable to build rust, status: ${JSON.stringify(cargoBuildResult)}`);
    }
}

function buildJs(projectRoot: string) {
    console.log("Building js...")
    const npmRunResult = spawnSync('npm', ['run', 'build'], { stdio: "inherit", cwd: projectRoot });

    if (npmRunResult.status !== 0) {
        throw new Error(`Unable to build js, status: ${JSON.stringify(npmRunResult)}`);
    }
}

function getProjectRoot(): string {
    const projectRoot = path.resolve(process.cwd(), '..', '..');
    console.log("Project root: " + projectRoot)
    return projectRoot
}

async function makeRepoChanges(projectRoot: string): Promise<{ releaseNotes: string; newVersion: number; }> {
    const git = simpleGit(projectRoot);

    console.log("Reading version file...")
    const oldVersion = await readVersion(projectRoot)

    const newVersion = oldVersion + 1;

    console.log("Writing version file...")
    await writeVersion(projectRoot, newVersion)

    console.log("Reading changelog file...")
    const changelogFilePath = path.join(projectRoot, "CHANGELOG.md");
    const changelogRaw = await readFile(changelogFilePath, { encoding: "utf-8" });

    const notesForCurrentRelease: string[] = []
    const newChangelog: string[] = []

    let section: "before" | "unreleased" | "after" = "before";
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
        const npmVersionResult = spawnSync('npm', ['version', `0.${newVersion}.0`], { stdio: "inherit", cwd: packageDir })

        if (npmVersionResult.status !== 0) {
            throw new Error(`Unable to run npm version, status: ${JSON.stringify(npmVersionResult)}`);
        }
    }

    console.log("Bump version for deno subproject...")
    const denoProjectPath = path.join(projectRoot, "js", "deno");
    bumpNpmPackage(denoProjectPath)

    console.log("Bump version for api subproject...")
    const apiProjectPath = path.join(projectRoot, "js", "api");
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

    return {
        newVersion,
        releaseNotes: notesForCurrentRelease.join('\n'),
    }
}

function packageForLinux(projectRoot: string, arch: string): { filePath: string; fileName: string } {
    const releaseDirPath = path.join(projectRoot, 'target', arch, 'release');
    const executableFileName = 'gauntlet';
    const archiveFileName = "gauntlet-x86_64-linux.tar.gz"
    const archiveFilePath = path.join(releaseDirPath, archiveFileName);

    const tarResult = spawnSync(`tar`, ['-czvf', archiveFileName, executableFileName], {
        stdio: "inherit",
        cwd: releaseDirPath
    })

    if (tarResult.status !== 0) {
        throw new Error(`Unable to package for linux, status: ${JSON.stringify(tarResult)}`);
    }

    return {
        filePath: archiveFilePath,
        fileName: archiveFileName
    }
}

async function packageForMacos(projectRoot: string, arch: string): Promise<{ filePath: string; fileName: string }> {
    const releaseDirPath = path.join(projectRoot, 'target', arch, 'release');
    const sourceExecutableFilePath = path.join(releaseDirPath, 'gauntlet');
    const outFileName = "gauntlet-aarch64-macos.dmg"
    const outFilePath = path.join(releaseDirPath, outFileName);
    const sourceInfoFilePath = path.join(projectRoot, 'assets', 'Info.plist');
    const sourceIconFilePath = path.join(projectRoot, 'assets', 'AppIcon.icns');

    const bundleDir = path.join(releaseDirPath, 'Gauntlet.app');
    const contentsDir = path.join(bundleDir, 'Contents');
    const macosContentsDir = path.join(contentsDir, 'MacOS');
    const resourcesContentsDir = path.join(contentsDir, 'Resources');
    const targetExecutableFilePath = path.join(macosContentsDir, 'Gauntlet');
    const targetInfoFilePath = path.join(contentsDir, 'Info.plist');
    const targetIconFilePath = path.join(resourcesContentsDir, 'AppIcon.icns');

    const dmgBackground = path.join(projectRoot, 'assets', 'dmg-background.png');

    const version = await readVersion(projectRoot)

    mkdirSync(bundleDir)
    mkdirSync(contentsDir)
    mkdirSync(macosContentsDir)
    mkdirSync(resourcesContentsDir)

    copyFileSync(sourceExecutableFilePath, targetExecutableFilePath)
    copyFileSync(sourceInfoFilePath, targetInfoFilePath)
    copyFileSync(sourceIconFilePath, targetIconFilePath)

    const infoSource = readFileSync(targetInfoFilePath, 'utf8');
    const infoResult = infoSource.replace('__VERSION__', `${version}.0.0`);
    writeFileSync(targetInfoFilePath, infoResult,'utf8');

    const createDmgResult = spawnSync(`create-dmg`, [
        '--volname', 'Gauntlet Installer',
        '--window-size', '660', '400',
        '--background', dmgBackground,
        '--icon', "Gauntlet.app", '180', '170',
        '--icon-size', '160',
        '--app-drop-link', '480', '170',
        '--hide-extension', 'Gauntlet.app',
        outFileName,
        bundleDir
    ], {
        stdio: "inherit",
        cwd: releaseDirPath
    })

    if (createDmgResult.status !== 0) {
        throw new Error(`Unable to package for macos, status: ${JSON.stringify(createDmgResult)}`);
    }

    return {
        filePath: outFilePath,
        fileName: outFileName
    }
}

function publishNpmPackage(projectRoot: string) {
    console.log("Publishing npm deno package...")
    const denoProjectPath = path.join(projectRoot, "js", "deno");
    const denoNpmPublish = spawnSync('npm', ['publish'], { stdio: "inherit", cwd: denoProjectPath })

    if (denoNpmPublish.status !== 0) {
        throw new Error(`Unable to publish deno package, status: ${JSON.stringify(denoNpmPublish)}`);
    }

    console.log("Publishing npm api package...")
    const apiProjectPath = path.join(projectRoot, "js", "api");
    const apiNpmPublish = spawnSync('npm', ['publish'], { stdio: "inherit", cwd: apiProjectPath })

    if (apiNpmPublish.status !== 0) {
        throw new Error(`Unable to publish api package, status: ${JSON.stringify(apiNpmPublish)}`);
    }
}

async function createRelease(newVersion: number, releaseNotes: string) {
    const octokit = getOctokit();

    console.log("Creating github release...")

    await octokit.rest.repos.createRelease({
        ...getGithubRepo(),
        tag_name: `v${newVersion}`,
        target_commitish: 'main',
        name: `v${newVersion}`,
        body: releaseNotes,
        draft: true
    });
}

async function addFileToRelease(projectRoot: string, filePath: string, fileName: string) {
    // this is old version because actions/checkout@v4 clones the ref
    // which triggered the workflow and not the latest which has updated version file
    const oldVersion = await readVersion(projectRoot)

    const newVersion = oldVersion + 1;

    const octokit = getOctokit();

    const response = await octokit.rest.repos.getReleaseByTag({
        ...getGithubRepo(),
        tag: `v${newVersion}`,
    });

    const fileBuffer = await readFile(filePath);

    console.log("Uploading executable as github release asset...")
    await octokit.rest.repos.uploadReleaseAsset({
        ...getGithubRepo(),
        release_id: response.data.id,
        origin: response.data.upload_url,
        name: fileName,
        headers: {
            "Content-Type": "application/octet-stream",
        },
        data: fileBuffer as any,
    })
}

function getOctokit() {
    return new Octokit({
        auth: process.env.GITHUB_TOKEN,
    })
}
function getGithubRepo() {
    return {
        owner: 'project-gauntlet',
        repo: 'gauntlet',
    }
}

async function readVersion(projectRoot: string): Promise<number> {
    const versionFilePath = path.join(projectRoot, "VERSION");
    const versionRaw = await readFile(versionFilePath, { encoding: "utf-8" });
    return Number(versionRaw.trim())
}

async function writeVersion(projectRoot: string, version: number) {
    const versionFilePath = path.join(projectRoot, "VERSION");
    await writeFile(versionFilePath, `${version}`)
}