import { ReactElement } from "react";
import { Grid } from "@project-gauntlet/api/components";

export default function Main(): ReactElement {
    const content = (
        <Grid.Item.Content>
            <Grid.Item.Content.Paragraph>
                Test
            </Grid.Item.Content.Paragraph>
        </Grid.Item.Content>
    );
    
    return (
        <Grid onItemFocusChange={(id: string | undefined) => console.log("onItemFocusChange", id)}>
            <Grid.Item id="adarian" title="Adarian">{content}</Grid.Item>
            <Grid.Item id="aruzan" title="Aruzan">{content}</Grid.Item>
            <Grid.Item id="blutopian" title="Blutopian">{content}</Grid.Item>
            <Grid.Item id="caphex" title="Caphex">{content}</Grid.Item>
            <Grid.Item id="condluran" title="Condluran">{content}</Grid.Item>
            <Grid.Item id="frozian" title="Frozian">{content}</Grid.Item>
            <Grid.Item id="evereni" title="Evereni">{content}</Grid.Item>
            <Grid.Item id="ezaraa" title="Ezaraa">{content}</Grid.Item>
            <Grid.Item id="houk" title="Houk">{content}</Grid.Item>
            <Grid.Item id="inleshat" title="Inleshat">{content}</Grid.Item>
        </Grid>
    )
}
