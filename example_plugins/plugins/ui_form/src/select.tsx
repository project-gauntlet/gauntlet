import { ReactElement } from 'react';
import { Form } from "@project-gauntlet/api/components";

export default function SelectExample(): ReactElement {
    return (
        <Form>
            <Form.Select label="Species" value="human">
                <Form.Select.Item value="human">Human</Form.Select.Item>
                <Form.Select.Item value="jawa">Jawa</Form.Select.Item>
                <Form.Select.Item value="hutt">Hutt</Form.Select.Item>
                <Form.Select.Item value="twi'lek">Twi'lek</Form.Select.Item>
                <Form.Select.Item value="wookie">Wookie</Form.Select.Item>
                <Form.Select.Item value="kaminoan">Kaminoan</Form.Select.Item>
            </Form.Select>
        </Form>
    );
};

