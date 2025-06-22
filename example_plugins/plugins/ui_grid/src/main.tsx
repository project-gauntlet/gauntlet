import { ReactElement } from "react";
import { Grid } from "@project-gauntlet/api/components";

const items = [
    {
        title: "Naboo",
        image: "https://static.wikia.nocookie.net/star-wars-canon/images/2/24/NabooFull-SW.png/revision/latest/scale-to-width-down/150?cb=20151218205422"
    },
    {
        title: "Ryloth",
        image: "https://static.wikia.nocookie.net/star-wars-canon/images/b/b7/Ryloth_Rebels.png/revision/latest/scale-to-width-down/150?cb=20161103040944"
    },
    {
        title: "Tatooine",
        image: "https://static.wikia.nocookie.net/star-wars-canon/images/b/b0/Tatooine_TPM.png/revision/latest/scale-to-width-down/150?cb=20151124205032"
    },
    {
        title: "Dagobah",
        image: "https://static.wikia.nocookie.net/star-wars-canon/images/4/48/Dagobah_ep3.jpg/revision/latest/scale-to-width-down/150?cb=20161103221846"
    },
    {
        title: "Endor",
        image: "https://static.wikia.nocookie.net/star-wars-canon/images/9/96/Endor-DB.png/revision/latest/scale-to-width-down/150?cb=20160711234205"
    },
    {
        title: "Dathomir",
        image: "https://static.wikia.nocookie.net/starwars/images/3/34/DathomirJFO.jpg/revision/latest/scale-to-width-down/150?cb=20200222032237"
    },
    {
        title: "Dantooine",
        image: "https://static.wikia.nocookie.net/starwars/images/9/9b/Dantooine-RebuildingTheResistance.jpg/revision/latest/scale-to-width-down/150?cb=20200120190043"
    },
]

export default function MainExample(): ReactElement {
    return (
        <Grid>
            {items.map(value => (
                <Grid.Item id={value.title} key={value.title} title={value.title}>
                    <Grid.Item.Content>
                        <Grid.Item.Content.Image source={{ url: value.image }}/>
                    </Grid.Item.Content>
                </Grid.Item>
            ))}
        </Grid>
    )
}
