import { GeneratorContext } from "@project-gauntlet/api/helpers";

export default async function EntrypointGenerator({ add, remove }: GeneratorContext): Promise<void | (() => void)>  {
    const id = "generated";
    const path = "/tmp/gauntlet-example";

    await Deno.create(path)
    add(id, generatedItem("default"))

    const watcher = Deno.watchFs(path);

    (async () => {
        for await (const _event of watcher) {
            try {
                const value = await Deno.readTextFile(path);
                add(id, generatedItem(value))
            } catch (err) {
                remove(id)
            }
        }
    })();

    return () => {
        watcher.close()
    }
}

function generatedItem(value: string) {
    return {
        name: `Generated Command - ${value}`,
        actions: [
            {
                label: "Run the Gauntlet",
                run: () => {
                    console.log('Running the Gauntlet...')
                }
            }
        ]
    };
}
