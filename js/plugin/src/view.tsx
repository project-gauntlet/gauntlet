import React, {useState} from 'react';

declare global {
    namespace JSX {
        interface IntrinsicElements {
            box: {}
            button1: { onClick?: () => void, children: string }
            // TODO remove default html IntrinsicElements
        }
    }
}

const View: React.FC = (): JSX.Element => {

    const [count, setCount] = useState(0);

    return (
        <box>
            test
            <box>
                {count}
            </box>
            <box>You clicked {count} times</box>
            <button1 onClick={() => {
                console.log("test events " + count)
                setCount(count + 1);
            }}>
                Click me
            </button1>
        </box>
    );
};

export default View;