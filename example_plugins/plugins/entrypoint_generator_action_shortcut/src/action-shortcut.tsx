import { GeneratorContext } from "@project-gauntlet/api/helpers";

type PluginGeneratorContext = GeneratorContext<{ testBool: boolean }, { testStr: string }>;

export default function EntrypointGenerator({ add }: PluginGeneratorContext): void {
    add('generated', {
        name: 'Generated Command',
        actions: [
            {
                label: "Primary Action", // Executed when Enter is pressed
                run: () => {
                    console.log('Running the Gauntlet... - Primary Action')
                }
            },
            {
                label: "Secondary Action", // Executed when Shift+Enter is pressed
                run: () => {
                    console.log('Running the Gauntlet... - Secondary Action')
                }
            },
            {
                ref: "anotherAction", // Executed when pressing shortcut specified in Plugin Manifest
                label: "Another Action",
                run: () => {
                    console.log('Running the Gauntlet... - Another Action')
                }
            }
        ],
    });
}
