import { assetData } from "@project-gauntlet/api/helpers";

export default async function Command() {
    const data = await assetData("masterpiece.txt");
    const decodedData = new TextDecoder().decode(data);
    console.log(decodedData)
}
