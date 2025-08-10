import { ReactElement, useState } from "react";
import { Action, ActionPanel, Grid } from "@project-gauntlet/api/components";

export default function Main(): ReactElement {
    const [id, setId] = useState<string | null>(null);

    const content = (id: string) => (
        <Grid.Item.Content>
            <Grid.Item.Content.Paragraph>
                {id}
            </Grid.Item.Content.Paragraph>
        </Grid.Item.Content>
    );

    return (
        <Grid
            onItemFocusChange={setId}
            focusedItemId={id}
            actions={
                <ActionPanel>
                    <Action
                        label={`Focused: ${id}`}
                        onAction={(id) => {
                            console.log(id)
                            setId("condluran")
                        }}
                    />
                </ActionPanel>
            }
        >
            <Grid.Item id="adarian" title="Adarian">{content("adarian")}</Grid.Item>
            <Grid.Item id="aruzan" title="Aruzan">{content("aruzan")}</Grid.Item>
            <Grid.Item id="blutopian" title="Blutopian">{content("blutopian")}</Grid.Item>
            <Grid.Item id="caphex" title="Caphex">{content("caphex")}</Grid.Item>
            <Grid.Item id="condluran" title="Condluran">{content("condluran")}</Grid.Item>
            <Grid.Item id="frozian" title="Frozian">{content("frozian")}</Grid.Item>
            <Grid.Item id="evereni" title="Evereni">{content("evereni")}</Grid.Item>
            <Grid.Item id="ezaraa" title="Ezaraa">{content("ezaraa")}</Grid.Item>
            <Grid.Item id="houk" title="Houk">{content("houk")}</Grid.Item>
            <Grid.Item id="inleshat" title="Inleshat">{content("inleshat")}</Grid.Item>
        </Grid>
    )
}
