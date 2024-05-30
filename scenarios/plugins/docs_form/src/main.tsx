import { ReactElement } from 'react';
import { Form } from "@project-gauntlet/api/components";

export default function Main(): ReactElement {
    return (
        <Form>
            <Form.Select
                label={"Food"}
                value={"burger"}
            >
                <Form.Select.Item value={"burger"}>Burger</Form.Select.Item>
            </Form.Select>
            <Form.TextField
                label={"Bun"}
                onChange={value => {
                    console.log(`value: ${value}`)
                }}
            />
            <Form.TextField
                label={"Meat"}
                value={"Chicken"}
                onChange={value => {
                    console.log(`value: ${value}`)
                }}
            />
            <Form.Checkbox
                label={"Cheese"}
                value={true}
                onChange={value => {
                    console.log(`value: ${value}`)
                }}
            />
            <Form.TextField
                label={"Toppings"}
                onChange={value => {
                    console.log(`value: ${value}`)
                }}
            />
            <Form.TextField
                label={"Condiments"}
                onChange={value => {
                    console.log(`value: ${value}`)
                }}
            />
            <Form.Separator/>
            <Form.DatePicker
                label={"Date"}
                value={"2024-03-22"}
                onChange={value => {
                    console.log(`value: ${value}`)
                }}
            />
        </Form>
    );
};

