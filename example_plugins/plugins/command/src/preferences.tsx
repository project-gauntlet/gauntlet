import { CommandContext } from "@project-gauntlet/api/helpers";

type PluginCommandContext = CommandContext<{ testBool: boolean }, {testStr: string }>;

export default async function Command({ pluginPreferences, entrypointPreferences }: PluginCommandContext) {
    console.log(pluginPreferences.testBool);
    console.log(entrypointPreferences.testStr);
}
