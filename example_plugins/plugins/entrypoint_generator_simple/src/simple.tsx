import { GeneratorContext } from "@project-gauntlet/api/helpers";
import { ReactElement } from "react";
import { List } from "@project-gauntlet/api/components";

function ListView(): ReactElement {
    return (
        <List>
            <List.Item id="example-item" title="Example Item"/>
        </List>
    )
}

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
        ]
    })

    add('generated', {
        name: 'Generated View',
        actions: [
            {
                label: "Open generated view",
                view: () => <ListView/>
            }
        ]
    })
}
