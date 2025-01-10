import { GeneratorProps, showHud } from "@project-gauntlet/api/helpers";
import { ReactElement } from "react";
import { List } from "@project-gauntlet/api/components";

function ListView(): ReactElement {
    return (
        <List>
            <List.Item id="test-item" title={"Test Item"}/>
        </List>
    )
}


export default function EntrypointGenerator({ add, remove: _ }: GeneratorProps): void {
    add('generated-test-1', {
        name: 'Generated Item 1',
        actions: [
            {
                label: "Run Generated Item 1",
                run: () => {
                    new Promise(() => {
                        throw new Error("gen")
                    })

                    console.log('generated-test-1')
                }
            }
        ]
    })

    add('generated-test-2', {
        name: 'Generated Item 2',
        actions: [
            {
                label: "Run Generated Item 2",
                run: () => {
                    console.log('generated-test-2')

                    sessionStorage.setItem("test", "test")
                    console.dir(sessionStorage.getItem("test"))

                    localStorage.setItem("test", "test")
                    console.dir(localStorage.getItem("test"))
                },
            },
            {
                label: "Test 1",
                run: () => {
                    console.log('generated-action-1')
                }
            },
            {
                ref: "testGeneratedAction1",
                label: "Test 2",
                run: () => {
                    console.log('generated-action-2')
                }
            }
        ]
    })

    add('generated-test-3', {
        name: 'Generated Item 3',
        actions: [
            {
                label: "Run Generated Item 3",
                run: () => {
                    showHud("HUD test display")
                    console.log('generated-test-3')
                },
            }
        ]
    })

    add('generated-test-4', {
        name: 'Generated Item 4',
        actions: [
            {
                label: "Run Generated Item 4",
                view: () => <ListView/>
            }
        ],
        accessories: [
            {
                text: "1 window open"
            }
        ],
    })
}
