import { ReactElement } from "react";
import { Grid } from "@project-gauntlet/api/components";

const items = [
    "C-3PO",
    "R2-D2",
    "BB-8",
    "IG-88",
    "D-O",
    "C1-10P",
]

export default function Main(): ReactElement {
    return (
        <Grid>
            {items.map(value => (
                <Grid.Item key={value}>
                    <Grid.Item.Content>
                        <Grid.Item.Content.CodeBlock>
                            {value}
                        </Grid.Item.Content.CodeBlock>
                    </Grid.Item.Content>
                </Grid.Item>
            ))}
        </Grid>
    )
}