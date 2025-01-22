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

export default function ContentParagraphExample(): ReactElement {
    return (
        <Grid>
            {items.map(value => (
                <Grid.Item id={value} key={value}>
                    <Grid.Item.Content>
                        <Grid.Item.Content.Paragraph>
                            {value}
                        </Grid.Item.Content.Paragraph>
                    </Grid.Item.Content>
                </Grid.Item>
            ))}
        </Grid>
    )
}