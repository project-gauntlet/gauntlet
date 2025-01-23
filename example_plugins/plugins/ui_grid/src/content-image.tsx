import { ReactElement } from "react";
import { Grid } from "@project-gauntlet/api/components";

export default function ContentImageExample(): ReactElement {
    return (
        <Grid>
                <Grid.Item id="tatooine" title="Tatooine">
                    <Grid.Item.Content>
                        <Grid.Item.Content.Image source={{ url: "https://static.wikia.nocookie.net/star-wars-canon/images/b/b0/Tatooine_TPM.png/revision/latest/scale-to-width-down/150?cb=20151124205032" }}/>
                    </Grid.Item.Content>
                </Grid.Item>
        </Grid>
    )
}