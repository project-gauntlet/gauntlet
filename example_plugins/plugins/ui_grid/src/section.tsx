import { ReactElement } from "react";
import { Grid } from "@project-gauntlet/api/components";

const theBlade1 = "https://static.wikia.nocookie.net/starwars/images/a/a4/The-Blade-1-final-cover.jpg/revision/latest/scale-to-width-down/150?cb=20221215195606"
const theBlade2 = "https://static.wikia.nocookie.net/starwars/images/f/fd/The-Blade-2-Final-Cover.jpg/revision/latest/scale-to-width-down/150?cb=20230120033002"

const vader1 = "https://static.wikia.nocookie.net/starwars/images/9/9a/Darth_VaderDark_Lord_of_the_Sith.jpg/revision/latest/scale-to-width-down/150?cb=20190223230434"

export default function SectionExample(): ReactElement {
    return (
        <Grid>
            <Grid.Section title="The High Republic">
                <Grid.Item id="the-blade-1" title="The Blade 1">
                    <Grid.Item.Content>
                        <Grid.Item.Content.Image source={{ url: theBlade1 }}/>
                    </Grid.Item.Content>
                </Grid.Item>
                <Grid.Item id="the-blade-2" title="The Blade 2">
                    <Grid.Item.Content>
                        <Grid.Item.Content.Image source={{ url: theBlade2 }}/>
                    </Grid.Item.Content>
                </Grid.Item>
            </Grid.Section>
            <Grid.Section title="Darth Vader">
                <Grid.Item id="darth-vader-1" title="Darth Vader 1">
                    <Grid.Item.Content>
                        <Grid.Item.Content.Image source={{ url: vader1 }}/>
                    </Grid.Item.Content>
                </Grid.Item>
            </Grid.Section>
        </Grid>
    )
}
