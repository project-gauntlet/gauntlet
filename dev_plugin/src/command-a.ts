export default function Command() {
    const env = Deno.env.get("LD_LIBRARY_PATH");

    console.log("LD_LIBRARY_PATH:", env);

    const command = new Deno.Command("echo", {
        args: ["test"],
        env: {
            LD_LIBRARY_PATH: ""
        }
    });

    const child = command.outputSync();

    const stdout = new TextDecoder().decode(child.stdout);

    console.dir(stdout)

    const systemMemoryInfo = Deno.systemMemoryInfo();

    console.dir(systemMemoryInfo)
}
