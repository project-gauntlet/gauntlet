import { ReactElement, useState } from 'react';
import { Form } from "@project-gauntlet/api/components";

export default function DetailView(): ReactElement {

    const [checked, setChecked] = useState(true);
    const [password, setPassword] = useState<string | undefined>("controlled password");
    const [text, setText] = useState<string | undefined>("controlled text");
    const [selected, setSelected] = useState<string | undefined>("default_selected_item");

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
            <Form.Select
                onChange={value => {
                    console.log(`uncontrolled value: ${value}`)
                }}
            >
                <Form.Select.Item value={"select_item_1"}>Select Item 1</Form.Select.Item>
                <Form.Select.Item value={"select_item_2"}>Select Item 2</Form.Select.Item>
                <Form.Select.Item value={"select_item_3"}>Select Item 3</Form.Select.Item>
                <Form.Select.Item value={"select_item_4"}>Select Item 4</Form.Select.Item>
            </Form.Select>
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
            <Form.Select
                value={selected}
                onChange={value => {
                    setSelected(value)
                    console.log(`controlled value: ${value}`)
                }}
            >
                <Form.Select.Item value={"select_item_1"}>Select Item 1</Form.Select.Item>
                <Form.Select.Item value={"select_item_2"}>Select Item 2</Form.Select.Item>
                <Form.Select.Item value={"default_selected_item"}>Default Select Item</Form.Select.Item>
                <Form.Select.Item value={"select_item_4"}>Select Item 4</Form.Select.Item>
            </Form.Select>
            <Form.DatePicker
                value={"2024-03-22"}
                onChange={value => {
                    console.log(`controlled value: ${value}`)
                }}
            />
        </Form>
    );
};

