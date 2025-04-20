import { GeneratorContext } from "@project-gauntlet/api/helpers";

export default async function EntrypointGenerator({ add }: GeneratorContext): Promise<void> {

    const response = await fetch("https://img.icons8.com/?size=32&id=21276&format=png");
    const arrayBuffer = await response.bytes();

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
        icon: arrayBuffer
    });
}
