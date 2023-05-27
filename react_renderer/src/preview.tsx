import React, {useState} from 'react';

const Preview: React.FC = (): JSX.Element => {

    const [count, setCount] = useState(0);

    return (
        <box>
            <box>You clicked {count} times</box>
            <button1 onClick={() => setCount(count + 1)}>
                Click me
            </button1>
        </box>
    );
};

export default Preview;