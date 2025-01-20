import { ReactElement, useState } from "react";
import { Grid } from "@project-gauntlet/api/components";

const results = [
    "Disturbances in the Force",
    "Bounty hunters",
    "Astromech droids",
    "Celestials and their technology",
    "What happened on holidays?",
    "Ahsoka Tano",
    "Mandalorian Culture"
]

export default function Main(): ReactElement {
    const [searchText, setSearchText] = useState<string | undefined>("");

    return (
        <Grid>
            <Grid.SearchBar placeholder="What knowledge do you seek...?"
                            value={searchText}
                            onChange={setSearchText}
            />
            {results
                .filter(value => !searchText ? true : value.toLowerCase().includes(searchText))
                .map(value => (
                    <Grid.Item id={value}>
                        <Grid.Item.Content>
                            <Grid.Item.Content.Paragraph>
                                {value}
                            </Grid.Item.Content.Paragraph>
                        </Grid.Item.Content>
                    </Grid.Item>
                ))
            }
        </Grid>
    )
}
