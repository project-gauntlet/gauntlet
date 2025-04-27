import { Clipboard } from "@project-gauntlet/api/helpers";
import { ReactElement } from "react";
import { ActionPanel, List } from "@project-gauntlet/api/components";

export default function View(): ReactElement {
    return (
        <List actions={
            <ActionPanel>
                <ActionPanel.Action label={"Run"} onAction={async (id) => {
                    switch (id) {
                        case "clear": {
                            await Clipboard.clear();
                            break;
                        }
                        case "read": {
                            const data = await Clipboard.read();
                            console.log(Deno.inspect(data));
                            break;
                        }
                        case "read-text": {
                            const data = await Clipboard.readText();
                            console.log(Deno.inspect(data));
                            break;
                        }
                        case "write": {
                            const res = await fetch("https://raw.githubusercontent.com/project-gauntlet/gauntlet/refs/heads/main/docs/logo.png");
                            const blob = await res.blob();
                            const image = await blob.arrayBuffer();
                            await Clipboard.write({ "image/png": image });
                            break;
                        }
                        case "write-text": {
                            await Clipboard.writeText("Gauntlet Example");
                            break;
                        }
                    }

                }}/>
            </ActionPanel>
        }>
            <List.Item id={"clear"} title={"Clear"}/>
            <List.Item id={"read"} title={"Read"}/>
            <List.Item id={"read-text"} title={"Read Text"}/>
            <List.Item id={"write"} title={"Write"}/>
            <List.Item id={"write-text"} title={"Write Text"}/>
        </List>
    )
}
