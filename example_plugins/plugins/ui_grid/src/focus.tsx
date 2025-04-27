import { ReactElement, useState } from "react";
import { Grid } from "@project-gauntlet/api/components";

export default function FocusExample(): ReactElement {
    const [id, setId] = useState<string | undefined>(undefined);

    const content = (text: string) => (
        <Grid.Item.Content>
            <Grid.Item.Content.Paragraph>
                {text}
            </Grid.Item.Content.Paragraph>
        </Grid.Item.Content>
    );

    return (
        <Grid
            onItemFocusChange={setId}
        >
            <Grid.Section title={"Focused: " + id}>
                <Grid.Item id="adarian">{content("Adarian")}</Grid.Item>
                <Grid.Item id="aruzan">{content("Aruzan")}</Grid.Item>
                <Grid.Item id="blutopian">{content("Blutopian")}</Grid.Item>
                <Grid.Item id="caphex">{content("Caphex")}</Grid.Item>
                <Grid.Item id="condluran">{content("Condluran")}</Grid.Item>
                <Grid.Item id="frozian">{content("Frozian")}</Grid.Item>
                <Grid.Item id="evereni">{content("Evereni")}</Grid.Item>
                <Grid.Item id="ezaraa">{content("Ezaraa")}</Grid.Item>
                <Grid.Item id="houk">{content("Houk")}</Grid.Item>
                <Grid.Item id="inleshat">{content("Inleshat")}</Grid.Item>
            </Grid.Section>
        </Grid>
    )
}
