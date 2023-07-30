import lowerCase from "lodash/lowerCase";

export default function View(): JSX.Element {
    return (
        <box>
            test {lowerCase("events")}
        </box>
    );
};

