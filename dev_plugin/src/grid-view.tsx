import { Grid, IconAccessory, Icons, List, TextAccessory } from "@project-gauntlet/api/components";
import { ReactElement, useState } from "react";
import { useStorage } from "@project-gauntlet/api/hooks";

export default function GridView(): ReactElement {
    const numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19];

    const [val1, setValue1] = useStorage("grid-view-test-1", undefined);
    const [val2, setValue2] = useStorage("grid-view-test-2", { " test": "test" });
    const [val3, setValue3] = useStorage("grid-view-test-3", "");
    const [val4, setValue4] = useStorage<string>("grid-view-test-4", "");

    const [searchText, setSearchText] = useState<string | undefined>("");

    return (
        <Grid>
            <Grid.SearchBar
                placeholder={"Search something..."}
                value={searchText}
                onChange={setSearchText}
            />
            {
                numbers.map(value => {
                    const title = "Title " + value;

                    if (title.toLowerCase().includes(searchText?.toLowerCase() ?? "")) {
                        return (
                            <Grid.Item title={"Title " + value}>
                                <Grid.Item.Content>
                                    <Grid.Item.Content.Paragraph>
                                        Test Paragraph {value}
                                    </Grid.Item.Content.Paragraph>
                                </Grid.Item.Content>
                            </Grid.Item>
                        )
                    } else {
                        return undefined
                    }
                })
            }
            <Grid.Section title="Section 1">
                <Grid.Item title="Title Section 1 1">
                    <Grid.Item.Content>
                        <Grid.Item.Content.Paragraph>
                            Test Paragraph Section 1 1
                        </Grid.Item.Content.Paragraph>
                    </Grid.Item.Content>
                </Grid.Item>
            </Grid.Section>
            <Grid.Section title="Section 2">
                <Grid.Item title="Title Section 2 1">
                    <Grid.Item.Content>
                        <Grid.Item.Content.Paragraph>
                            Test Paragraph Section 2 1
                        </Grid.Item.Content.Paragraph>
                    </Grid.Item.Content>
                </Grid.Item>
                <Grid.Item
                    title="Title Section"
                    subtitle="Test subtitle"
                    accessory={<IconAccessory icon={Icons.Wallet} tooltip="Tooltip"/>}
                >
                    <Grid.Item.Content>
                        <Grid.Item.Content.Paragraph>
                            Test Paragraph Section 2 2
                        </Grid.Item.Content.Paragraph>
                    </Grid.Item.Content>
                </Grid.Item>
                <Grid.Item accessory={<IconAccessory icon={Icons.Sun} tooltip="Tooltip"/>}>
                    <Grid.Item.Content>
                        <Grid.Item.Content.Paragraph>
                            Test Paragraph Section 2 2
                        </Grid.Item.Content.Paragraph>
                    </Grid.Item.Content>
                </Grid.Item>
            </Grid.Section>
        </Grid>
    )
}
