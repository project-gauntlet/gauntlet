import { ReactElement } from "react";
import { Grid } from "@project-gauntlet/api/components";

const items = [
    "🥹",
    "🤣",
    "🥵",
    "🤕",
    "🫥",
    "🤬",
    "🥱",
    "🤮",
    "🙄",
    "🤠"
]

export default function MoreColumnsExample(): ReactElement {
    return (
        <Grid columns={8}>
            {items.map(value => (
                <Grid.Item id={value} key={value}>
                    <Grid.Item.Content>
                        <Grid.Item.Content.Paragraph>{value}</Grid.Item.Content.Paragraph>
                    </Grid.Item.Content>
                </Grid.Item>
            ))}
        </Grid>
    )
}