import { ReactElement, useState } from 'react';
import { Form } from "@project-gauntlet/api/components";

export default function DetailView(): ReactElement {

    const [checked, setChecked] = useState(true);
    const [password, setPassword] = useState<string | undefined>("controlled password");
    const [text, setText] = useState<string | undefined>("controlled text");

    return (
        <Form>
            {/* uncontrolled */}
            <Form.TextField
                onChange={value => {
                    console.log(`uncontrolled value: ${value}`)
                }}
            />
            <Form.PasswordField
                onChange={value => {
                    console.log(`uncontrolled value: ${value}`)
                }}
            />
            <Form.Checkbox
                onChange={value => {
                    console.log(`uncontrolled value: ${value}`)
                }}
            />
            <Form.Select/>
            <Form.DatePicker
                onChange={value => {
                    console.log(`uncontrolled value: ${value}`)
                }}
            />
            <Form.Separator/>
            {/* controlled */}
            <Form.TextField
                value={text}
                onChange={value => {
                    setText(value)
                    console.log(`controlled value: ${value}`)
                }}
            />
            <Form.PasswordField
                value={password}
                onChange={value => {
                    setPassword(value)
                    console.log(`controlled value: ${value}`)
                }}
            />
            <Form.Checkbox
                value={checked}
                onChange={value => {
                    setChecked(value)
                    console.log(`controlled value: ${value}`)
                }}
            />
            <Form.Select/>
            <Form.DatePicker
                value={"2024-03-22"}
                onChange={value => {
                    console.log(`controlled value: ${value}`)
                }}
            />
        </Form>
    );
};

