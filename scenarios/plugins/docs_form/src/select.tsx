import { ReactElement } from 'react';
import { Form } from "@project-gauntlet/api/components";

export default function Main(): ReactElement {
    return (
        // docs-code-segment:start
        <Form>
            <Form.Select label={"Food"} value={"burger"}>
                <Form.Select.Item value={"burger"}>Burger</Form.Select.Item>
                <Form.Select.Item value={"hot-dog"}>Hot Dog</Form.Select.Item>
                <Form.Select.Item value={"croissant"}>Croissant</Form.Select.Item>
                <Form.Select.Item value={"cookies"}>Cookies</Form.Select.Item>
                <Form.Select.Item value={"steak"}>Steak</Form.Select.Item>
                <Form.Select.Item value={"seafood"}>Seafood</Form.Select.Item>
            </Form.Select>
        </Form>
        // docs-code-segment:end
    );
};

