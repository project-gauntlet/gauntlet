import { ReactElement } from "react";
import { Grid } from "@project-gauntlet/api/components";

export default function ContentHorizontalBreak(): ReactElement {
    return (
        <Grid>
            <Grid.Item id="test">
                <Grid.Item.Content>
                    <Grid.Item.Content.Paragraph>
                        C-3PO
                    </Grid.Item.Content.Paragraph>
                    <Grid.Item.Content.HorizontalBreak/>
                    <Grid.Item.Content.Paragraph>
                        BB-8
                    </Grid.Item.Content.Paragraph>
                </Grid.Item.Content>
            </Grid.Item>
        </Grid>
    )
}