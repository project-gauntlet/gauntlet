import { GeneratorContext } from "@project-gauntlet/api/helpers";

type PluginGeneratorContext = GeneratorContext<{ testBool: boolean }, { testStr: string }>;

export default function EntrypointGenerator({ add, pluginPreferences, entrypointPreferences }: PluginGeneratorContext): void {
    add('generated', {
        name: 'Generated Command - ' + entrypointPreferences.testStr,
        actions: [
            {
                label: "Run the Gauntlet",
                run: () => {
                    console.log('Running the Gauntlet... ' + pluginPreferences.testBool)
                }
            }
        ],
    });
}
