import { ReactElement } from "react";
import { Grid } from "@project-gauntlet/api/components";

const theBlade1 = "https://static.wikia.nocookie.net/starwars/images/a/a4/The-Blade-1-final-cover.jpg/revision/latest/scale-to-width-down/150?cb=20221215195606"
const theBlade2 = "https://static.wikia.nocookie.net/starwars/images/f/fd/The-Blade-2-Final-Cover.jpg/revision/latest/scale-to-width-down/150?cb=20230120033002"
const theBlade3 = "https://static.wikia.nocookie.net/starwars/images/0/02/The-Blade-3-Final-Cover.jpg/revision/latest/scale-to-width-down/150?cb=20230227203337"
const theBlade4 = "https://static.wikia.nocookie.net/starwars/images/6/6c/The-Blade-4-Final-Cover.jpg/revision/latest/scale-to-width-down/150?cb=20230321223753"

const vader1 = "https://static.wikia.nocookie.net/starwars/images/9/9a/Darth_VaderDark_Lord_of_the_Sith.jpg/revision/latest/scale-to-width-down/150?cb=20190223230434"
const vader2 = "https://static.wikia.nocookie.net/starwars/images/2/2e/Darth_Vader_2_cover_art.jpg/revision/latest/scale-to-width-down/150?cb=20190223234228"
const vader3 = "https://static.wikia.nocookie.net/starwars/images/d/df/DarthVader2017-3.jpg/revision/latest/scale-to-width-down/150?cb=20190224013414"
const vader4 = "https://static.wikia.nocookie.net/starwars/images/c/c9/Darthvader-dlots-4-final.jpg/revision/latest/scale-to-width-down/150?cb=20190226024707"
const vader5 = "https://static.wikia.nocookie.net/starwars/images/a/ab/Darthvader-dlots-5.jpg/revision/latest/scale-to-width-down/150?cb=20170826121053"
const vader6 = "https://static.wikia.nocookie.net/starwars/images/2/20/DarthVader-DLotS--Solicitation.jpg/revision/latest/scale-to-width-down/150?cb=20171001165404"
const vader7 = "https://static.wikia.nocookie.net/starwars/images/f/fa/DarthVader2017-7.jpg/revision/latest/scale-to-width-down/150?cb=20190226233333"

export default function Main(): ReactElement {
    return (
        <Grid>
            <Grid.Section title="The High Republic">
                <Grid.Item id="the-blade-1" title="The Blade 1">
                    <Grid.Item.Content.Image source={{ url: theBlade1 }}/>
                </Grid.Item>
                <Grid.Item id="the-blade-2" title="The Blade 2">
                    <Grid.Item.Content.Image source={{ url: theBlade2 }}/>
                </Grid.Item>
                <Grid.Item id="the-blade-3" title="The Blade 3">
                    <Grid.Item.Content.Image source={{ url: theBlade3 }}/>
                </Grid.Item>
                <Grid.Item id="the-blade-4" title="The Blade 4">
                    <Grid.Item.Content.Image source={{ url: theBlade4 }}/>
                </Grid.Item>
            </Grid.Section>
            <Grid.Section title="Darth Vader">
                <Grid.Item id="darth-vader-1" title="Darth Vader 1">
                    <Grid.Item.Content.Image source={{ url: vader1 }}/>
                </Grid.Item>
                <Grid.Item id="darth-vader-2" title="Darth Vader 2">
                    <Grid.Item.Content.Image source={{ url: vader2 }}/>
                </Grid.Item>
                <Grid.Item id="darth-vader-3" title="Darth Vader 3">
                    <Grid.Item.Content.Image source={{ url: vader3 }}/>
                </Grid.Item>
                <Grid.Item id="darth-vader-4" title="Darth Vader 4">
                    <Grid.Item.Content.Image source={{ url: vader4 }}/>
                </Grid.Item>
                <Grid.Item id="darth-vader-5" title="Darth Vader 5">
                    <Grid.Item.Content.Image source={{ url: vader5 }}/>
                </Grid.Item>
                <Grid.Item id="darth-vader-6" title="Darth Vader 6">
                    <Grid.Item.Content.Image source={{ url: vader6 }}/>
                </Grid.Item>
                <Grid.Item id="darth-vader-7" title="Darth Vader 7">
                    <Grid.Item.Content.Image source={{ url: vader7 }}/>
                </Grid.Item>
            </Grid.Section>
        </Grid>
    )
}
