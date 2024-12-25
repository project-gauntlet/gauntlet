import { GeneratorProps, showHud } from "@project-gauntlet/api/helpers";

export default function CommandGenerator({ add, remove: _ }: GeneratorProps): void {
    add('generated-test-1', {
        name: 'Generated Item 1',
        fn: () => {
            new Promise(() => {
                throw new Error("gen")
            })

            console.log('generated-test-1')
        }
    })

    add('generated-test-2', {
        name: 'Generated Item 2',
        fn: () => {
            console.log('generated-test-2')

            sessionStorage.setItem("test", "test")
            console.dir(sessionStorage.getItem("test"))

            localStorage.setItem("test", "test")
            console.dir(localStorage.getItem("test"))
        },
        actions: [
            {
                label: "Test 1",
                fn: () => {
                    console.log('generated-action-1')
                }
            },
            {
                ref: "testGeneratedAction1",
                label: "Test 2",
                fn: () => {
                    console.log('generated-action-2')
                }
            }
        ]
    })

    add('generated-test-3', {
        name: 'Generated Item 3',
        fn: () => {
            showHud("HUD test display")
            console.log('generated-test-3')
        }
    })

    add('generated-test-4', {
        name: 'Generated Item 4',
        fn: () => {
            console.log('generated-test-4')
        },
        accessories: [
            {
                text: "1 window open"
            }
        ],
    })
}
