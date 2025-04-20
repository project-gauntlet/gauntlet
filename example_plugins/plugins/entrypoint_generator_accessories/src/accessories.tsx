import { GeneratorContext } from "@project-gauntlet/api/helpers";
import { Icons } from "@project-gauntlet/api/components";

export default function EntrypointGenerator({ add }: GeneratorContext): void {
    add('generated', {
        name: 'Generated Command',
        actions: [
            {
                label: "Run the Gauntlet",
                run: () => {
                    console.log('Running the Gauntlet...')
                }
            }
        ],
        accessories: [{ icon: Icons.Battery, text: "100 %" }]
    })
}
