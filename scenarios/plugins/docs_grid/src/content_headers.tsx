import { ReactElement } from "react";
import { Grid } from "@project-gauntlet/api/components";

export default function Main(): ReactElement {
    return (
        <Grid>
            <Grid.Item>
                <Grid.Item.Content>
                    <Grid.Item.Content.H1>
                        Episode I
                    </Grid.Item.Content.H1>
                </Grid.Item.Content>
            </Grid.Item>
            <Grid.Item>
                <Grid.Item.Content>
                    <Grid.Item.Content.H2>
                        Episode II
                    </Grid.Item.Content.H2>
                </Grid.Item.Content>
            </Grid.Item>
            <Grid.Item>
                <Grid.Item.Content>
                    <Grid.Item.Content.H3>
                        Episode III
                    </Grid.Item.Content.H3>
                </Grid.Item.Content>
            </Grid.Item>
            <Grid.Item>
                <Grid.Item.Content>
                    <Grid.Item.Content.H4>
                        Episode IV
                    </Grid.Item.Content.H4>
                </Grid.Item.Content>
            </Grid.Item>
            <Grid.Item>
                <Grid.Item.Content>
                    <Grid.Item.Content.H4>
                        Episode V
                    </Grid.Item.Content.H4>
                </Grid.Item.Content>
            </Grid.Item>
            <Grid.Item>
                <Grid.Item.Content>
                    <Grid.Item.Content.H4>
                        Episode VI
                    </Grid.Item.Content.H4>
                </Grid.Item.Content>
            </Grid.Item>
        </Grid>
    )
}