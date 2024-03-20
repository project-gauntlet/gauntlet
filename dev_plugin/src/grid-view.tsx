import { Grid } from "@project-gauntlet/api/components";
import { ReactElement } from "react";

export default function GridView(): ReactElement {
    const numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19];

    return (
        <Grid>
            {
                numbers.map(value => (
                    <Grid.Item id={"id" + value} title={"Title " + value}>
                        <Grid.Item.Content>
                            <Grid.Item.Content.Paragraph>
                                Test Paragraph {value}
                            </Grid.Item.Content.Paragraph>
                        </Grid.Item.Content>
                    </Grid.Item>
                ))
            }
            <Grid.Section title="Section 1">
                <Grid.Item id="id section 1 1" title="Title Section 1 1">
                    <Grid.Item.Content>
                        <Grid.Item.Content.Paragraph>
                            Test Paragraph Section 1 1
                        </Grid.Item.Content.Paragraph>
                    </Grid.Item.Content>
                </Grid.Item>
            </Grid.Section>
            <Grid.Section title="Section 2">
                <Grid.Item id="id section 2 1" title="Title Section 2 1">
                    <Grid.Item.Content>
                        <Grid.Item.Content.Paragraph>
                            Test Paragraph Section 2 1
                        </Grid.Item.Content.Paragraph>
                    </Grid.Item.Content>
                </Grid.Item>
                <Grid.Item id="id section 2 2" title="Title Section 2 2">
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
