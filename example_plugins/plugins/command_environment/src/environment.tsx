import { Environment } from "@project-gauntlet/api/helpers";

export default function Command() {
    console.log(Environment.gauntletVersion)
    console.log(Environment.isDevelopment)
    console.log(Environment.pluginDataDir)
    console.log(Environment.pluginCacheDir)
}
