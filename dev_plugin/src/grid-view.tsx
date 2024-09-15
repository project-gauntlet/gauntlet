import { Grid } from "@project-gauntlet/api/components";
import { ReactElement } from "react";
import { useStorage } from "@project-gauntlet/api/hooks";

export default function GridView(): ReactElement {
    const numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19];

    const [val1, setValue1] = useStorage("test", undefined);
    const [val2, setValue2] = useStorage("test", { " test": "test" });
    const [val3, setValue3] = useStorage("test", "");
    const [val4, setValue4] = useStorage<string>("test", "");

    return (
        <Grid>
            {
                numbers.map(value => (
                    <Grid.Item title={"Title " + value}>
                        <Grid.Item.Content>
                            <Grid.Item.Content.Paragraph>
                                Test Paragraph {value}
                            </Grid.Item.Content.Paragraph>
                        </Grid.Item.Content>
                    </Grid.Item>
                ))
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
                <Grid.Item title="Title Section 2 2">
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
