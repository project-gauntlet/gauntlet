import { ReactElement } from 'react';
import { Action, ActionPanel, Form } from "@project-gauntlet/api/components";

export default function MainExample(): ReactElement {
    return (
        <Form
            actions={
                <ActionPanel>
                    <Action label="Submit" onAction={() => {}}/>
                </ActionPanel>
            }
        >
            <Form.TextField label="First Name" value="Cassian" onChange={value => {
                console.log(`First Name: ${value}`)
            }}/>
            <Form.TextField label="Last Name" value="Andor" onChange={value => {
                console.log(`Last Name: ${value}`)
            }}/>
            <Form.Select
                label="Species"
                value="human"
                onChange={value => {
                console.log(`Last Name: ${value}`)
            }}>
                <Form.Select.Item value="human">Human</Form.Select.Item>
                <Form.Select.Item value="jawa">Jawa</Form.Select.Item>
                <Form.Select.Item value="hutt">Hutt</Form.Select.Item>
                <Form.Select.Item value="twi'lek">Twi'lek</Form.Select.Item>
            </Form.Select>

            <Form.Separator/>

            <Form.DatePicker
                label="Date"
                value={"2024-03-22"}
                onChange={value => {
                    console.log(`value: ${value}`)
                }}
            />
            <Form.Checkbox
                title="I acknowledge the Galactic Code and agree to abide by its regulations."
                value={false}
                onChange={value => {
                    console.log(`terms of service: ${value}`)
                }}
            />
        </Form>
    );
};

