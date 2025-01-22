import { ReactElement } from "react";
import { Grid } from "@project-gauntlet/api/components";

const alderaanImage = "https://static.wikia.nocookie.net/starwars/images/4/4a/Alderaan.jpg/revision/latest?cb=20061211013805"

export default function EmptyViewExample(): ReactElement {
    return (
        <Grid>
            <Grid.EmptyView title="Nothing here" description="But there was something" image={{ url: alderaanImage }}/>
        </Grid>
    )
}
