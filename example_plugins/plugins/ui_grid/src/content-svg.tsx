import { ReactElement } from "react";
import { Grid } from "@project-gauntlet/api/components";

const svgUrl = "https://upload.wikimedia.org/wikipedia/commons/8/84/Example.svg"

export default function ContentSvgExample(): ReactElement {
    return (
        <Grid>
            <Grid.Item id="svg">
                <Grid.Item.Content>
                    <Grid.Item.Content.Svg source={{ url: svgUrl }}/>
                </Grid.Item.Content>
            </Grid.Item>
        </Grid>
    )
}
