import ts from "typescript";
import { readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";

type Component = StandardComponent | RootComponent | TextPartComponent

type StandardComponent = {
    type: "standard",
    internalName: string,
    name: string,
    props: Property[],
    children: Children,
}

type RootComponent = {
    type: "root",
    internalName: string,
    children: RootChild[],
}

type TextPartComponent = {
    type: "text_part",
    internalName: string,
}

type Property = {
    name: string
    optional: boolean
    type: PropertyType
}
type PropertyType = TypeString | TypeNumber | TypeBoolean | TypeArray | TypeFunction
type Children = ChildrenMembers | ChildrenString | ChildrenNone | ChildrenStringOrMembers

type ChildrenMembers = {
    type: "members",
    members: ChildrenMember[]
}
type ChildrenStringOrMembers = {
    type: "string_or_members",
    component_internal_name: string,
    members: ChildrenMember[]
}
type ChildrenString = {
    type: "string"
    component_internal_name: string,
}
type ChildrenNone = {
    type: "none"
}

type ChildrenMember = {
    memberName: string,
    componentInternalName: string,
    componentName: string,
}

type RootChild = {
    componentInternalName: string,
    componentName: string,
}

type TypeString = {
    type: "string"
}
type TypeNumber = {
    type: "number"
}
type TypeBoolean = {
    type: "boolean"
}
type TypeArray = {
    type: "array"
    nested: PropertyType
}
type TypeFunction = {
    type: "function"
    arguments: Property[]
}

function generate(componentModelPath: string, outFile: string) {
    const content = readFileSync(componentModelPath).toString();
    const model = JSON.parse(content) as Component[]

    const resultFile = ts.createSourceFile("unused", "", ts.ScriptTarget.Latest, false, ts.ScriptKind.TSX);
    const printer = ts.createPrinter({ newLine: ts.NewLineKind.LineFeed });

    const result = printer.printNode(ts.EmitHint.Unspecified, makeComponents(model), resultFile);

    writeFileSync(outFile, result)
}

function makeComponents(modelInput: Component[]): ts.SourceFile {
    const model = modelInput.filter((component): component is StandardComponent => component.type === "standard");

    const imports = [
        ts.factory.createImportDeclaration(
            undefined,
            ts.factory.createImportClause(
                false,
                undefined,
                ts.factory.createNamedImports([
                    ts.factory.createImportSpecifier(
                        false,
                        undefined,
                        ts.factory.createIdentifier("FC")
                    ),
                    ts.factory.createImportSpecifier(
                        false,
                        undefined,
                        ts.factory.createIdentifier("JSXElementConstructor")
                    ),
                    ts.factory.createImportSpecifier(
                        false,
                        undefined,
                        ts.factory.createIdentifier("ReactElement")
                    ),
                    ts.factory.createImportSpecifier(
                        false,
                        undefined,
                        ts.factory.createIdentifier("ReactNode")
                    )
                ])
            ),
            ts.factory.createStringLiteral("react"),
            undefined
        )
    ];

    const publicDeclarations = [
        ts.factory.createTypeAliasDeclaration(
            [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
            ts.factory.createIdentifier("ElementParams"),
            [ts.factory.createTypeParameterDeclaration(
                undefined,
                ts.factory.createIdentifier("Comp"),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("FC"),
                    [ts.factory.createKeywordTypeNode(ts.SyntaxKind.AnyKeyword)]
                ),
                undefined
            )],
            ts.factory.createConditionalTypeNode(
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("Comp"),
                    undefined
                ),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("FC"),
                    [ts.factory.createInferTypeNode(ts.factory.createTypeParameterDeclaration(
                        undefined,
                        ts.factory.createIdentifier("Params"),
                        undefined,
                        undefined
                    ))]
                ),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("Params"),
                    undefined
                ),
                ts.factory.createKeywordTypeNode(ts.SyntaxKind.NeverKeyword)
            )
        ),
        ts.factory.createTypeAliasDeclaration(
            [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
            ts.factory.createIdentifier("Element"),
            [ts.factory.createTypeParameterDeclaration(
                undefined,
                ts.factory.createIdentifier("Comp"),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("FC"),
                    [ts.factory.createKeywordTypeNode(ts.SyntaxKind.AnyKeyword)]
                ),
                undefined
            )],
            ts.factory.createTypeReferenceNode(
                ts.factory.createIdentifier("ReactElement"),
                [
                    ts.factory.createTypeReferenceNode(
                        ts.factory.createIdentifier("ElementParams"),
                        [ts.factory.createTypeReferenceNode(
                            ts.factory.createIdentifier("Comp"),
                            undefined
                        )]
                    ),
                    ts.factory.createTypeReferenceNode(
                        ts.factory.createIdentifier("JSXElementConstructor"),
                        [ts.factory.createTypeReferenceNode(
                            ts.factory.createIdentifier("ElementParams"),
                            [ts.factory.createTypeReferenceNode(
                                ts.factory.createIdentifier("Comp"),
                                undefined
                            )]
                        )]
                    )
                ]
            )
        ),
        ts.factory.createTypeAliasDeclaration(
            [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
            ts.factory.createIdentifier("StringNode"),
            undefined,
            ts.factory.createUnionTypeNode([
                ts.factory.createKeywordTypeNode(ts.SyntaxKind.StringKeyword),
                ts.factory.createKeywordTypeNode(ts.SyntaxKind.NumberKeyword)
            ])
        ),
        ts.factory.createTypeAliasDeclaration(
            [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
            ts.factory.createIdentifier("EmptyNode"),
            undefined,
            ts.factory.createUnionTypeNode([
                ts.factory.createKeywordTypeNode(ts.SyntaxKind.BooleanKeyword),
                ts.factory.createLiteralTypeNode(ts.factory.createNull()),
                ts.factory.createKeywordTypeNode(ts.SyntaxKind.UndefinedKeyword)
            ])
        ),
        ts.factory.createTypeAliasDeclaration(
            [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
            ts.factory.createIdentifier("ElementComponent"),
            [ts.factory.createTypeParameterDeclaration(
                undefined,
                ts.factory.createIdentifier("Comp"),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("FC"),
                    [ts.factory.createKeywordTypeNode(ts.SyntaxKind.AnyKeyword)]
                ),
                undefined
            )],
            ts.factory.createUnionTypeNode([
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("Element"),
                    [ts.factory.createTypeReferenceNode(
                        ts.factory.createIdentifier("Comp"),
                        undefined
                    )]
                ),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("EmptyNode"),
                    undefined
                ),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("Iterable"),
                    [ts.factory.createTypeReferenceNode(
                        ts.factory.createIdentifier("ElementComponent"),
                        [ts.factory.createTypeReferenceNode(
                            ts.factory.createIdentifier("Comp"),
                            undefined
                        )]
                    )]
                )
            ])
        ),
        ts.factory.createTypeAliasDeclaration(
            [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
            ts.factory.createIdentifier("StringComponent"),
            undefined,
            ts.factory.createUnionTypeNode([
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("StringNode"),
                    undefined
                ),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("EmptyNode"),
                    undefined
                ),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("Iterable"),
                    [ts.factory.createTypeReferenceNode(
                        ts.factory.createIdentifier("StringComponent"),
                        undefined
                    )]
                )
            ])
        ),
        ts.factory.createTypeAliasDeclaration(
            [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
            ts.factory.createIdentifier("StringOrElementComponent"),
            [ts.factory.createTypeParameterDeclaration(
                undefined,
                ts.factory.createIdentifier("Comp"),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("FC"),
                    [ts.factory.createKeywordTypeNode(ts.SyntaxKind.AnyKeyword)]
                ),
                undefined
            )],
            ts.factory.createUnionTypeNode([
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("StringNode"),
                    undefined
                ),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("EmptyNode"),
                    undefined
                ),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("Element"),
                    [ts.factory.createTypeReferenceNode(
                        ts.factory.createIdentifier("Comp"),
                        undefined
                    )]
                ),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("Iterable"),
                    [ts.factory.createTypeReferenceNode(
                        ts.factory.createIdentifier("StringOrElementComponent"),
                        [ts.factory.createTypeReferenceNode(
                            ts.factory.createIdentifier("Comp"),
                            undefined
                        )]
                    )]
                )
            ])
        )
    ];



    const internalDeclarations = [
        ts.factory.createModuleDeclaration(
            [ts.factory.createToken(ts.SyntaxKind.DeclareKeyword)],
            ts.factory.createIdentifier("global"),
            ts.factory.createModuleBlock([ts.factory.createModuleDeclaration(
                undefined,
                ts.factory.createIdentifier("JSX"),
                ts.factory.createModuleBlock([ts.factory.createInterfaceDeclaration(
                    undefined,
                    ts.factory.createIdentifier("IntrinsicElements"),
                    undefined,
                    undefined,
                    model.map(component => {
                        return ts.factory.createPropertySignature(
                            undefined,
                            ts.factory.createComputedPropertyName(ts.factory.createStringLiteral(`gauntlet:${component.internalName}`)),
                            undefined,
                            ts.factory.createTypeLiteralNode(
                                makePropertyTypes(component)
                            )
                        )
                    })
                )]),
                ts.NodeFlags.Namespace | ts.NodeFlags.ExportContext | ts.NodeFlags.ContextFlags
            )]),
            ts.NodeFlags.ExportContext | ts.NodeFlags.GlobalAugmentation | ts.NodeFlags.ContextFlags
        )
    ]

    // abuse the fact that there is no space between multiline comment and content
    // is three a better way to add @internal tag to 'declare global'?
    for (const internalDeclaration of internalDeclarations) {
        ts.addSyntheticLeadingComment(internalDeclaration, ts.SyntaxKind.MultiLineCommentTrivia, "*@internal", true)
    }

    const components = model.flatMap(component => {

        const properties = component.props.map((prop) => (
            ts.factory.createJsxAttribute(
                ts.factory.createIdentifier(prop.name),
                ts.factory.createJsxExpression(
                    undefined,
                    ts.factory.createPropertyAccessExpression(
                        ts.factory.createIdentifier("props"),
                        ts.factory.createIdentifier(prop.name)
                    )
                )
            )
        ));

        if (component.children.type != "none") {
            properties.unshift(ts.factory.createJsxAttribute(
                ts.factory.createIdentifier("children"),
                ts.factory.createJsxExpression(
                    undefined,
                    ts.factory.createPropertyAccessExpression(
                        ts.factory.createIdentifier("props"),
                        ts.factory.createIdentifier("children")
                    )
                )
            ))
        }

        const componentFCType = ts.factory.createTypeReferenceNode(
            ts.factory.createIdentifier("FC"),
            properties.length === 0 ? [] : [
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier(`${component.name}Props`),
                    undefined
                )
            ]
        )

        let componentType: ts.TypeReferenceNode | ts.IntersectionTypeNode;
        if (component.children.type == "members" || component.children.type == "string_or_members") {
            componentType = ts.factory.createIntersectionTypeNode([
                componentFCType,
                ts.factory.createTypeLiteralNode(
                    component.children.members.map(member => {
                        return ts.factory.createPropertySignature(
                            undefined,
                            ts.factory.createIdentifier(member.memberName),
                            undefined,
                            ts.factory.createTypeQueryNode(
                                ts.factory.createIdentifier(member.componentName),
                                undefined
                            )
                        );
                    })
                )
            ]);
        } else {
            componentType = componentFCType;
        }


        let memberAssignments: ts.Statement[];
        switch (component.children.type) {
            case "string_or_members":
            case "members": {
                memberAssignments = component.children.members.map(member => {
                    return ts.factory.createExpressionStatement(ts.factory.createBinaryExpression(
                        ts.factory.createPropertyAccessExpression(
                            ts.factory.createIdentifier(component.name),
                            ts.factory.createIdentifier(member.memberName)
                        ),
                        ts.factory.createToken(ts.SyntaxKind.EqualsToken),
                        ts.factory.createIdentifier(member.componentName)
                    ))
                });
                break;
            }
            case "string": {
                memberAssignments = [];
                break;
            }
            case "none": {
                memberAssignments = [];
                break;
            }
            default: {
                throw new Error("unreachable")
            }
        }

        const interfaceProps = makePropertyTypes(component);

        const interfaceDeclaration: ts.InterfaceDeclaration[] = interfaceProps.length === 0 ? [] : [
            ts.factory.createInterfaceDeclaration(
                [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
                ts.factory.createIdentifier(`${component.name}Props`),
                undefined,
                undefined,
                interfaceProps
            )
        ];

        const propsParameter = properties.length === 0 ? [] : [
            ts.factory.createParameterDeclaration(
                undefined,
                undefined,
                ts.factory.createIdentifier("props"),
                undefined,
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier(`${component.name}Props`),
                    undefined
                ),
                undefined
            )
        ];

        return [
            ...interfaceDeclaration,
            ts.factory.createVariableStatement(
                [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
                ts.factory.createVariableDeclarationList(
                    [ts.factory.createVariableDeclaration(
                        ts.factory.createIdentifier(component.name),
                        undefined,
                        componentType,
                        ts.factory.createArrowFunction(
                            undefined,
                            undefined,
                            propsParameter,
                            ts.factory.createTypeReferenceNode(
                                ts.factory.createIdentifier("ReactNode"),
                                undefined
                            ),
                            ts.factory.createToken(ts.SyntaxKind.EqualsGreaterThanToken),
                            ts.factory.createBlock(
                                [
                                    ts.factory.createReturnStatement(
                                        ts.factory.createJsxSelfClosingElement(
                                            ts.factory.createJsxNamespacedName(
                                                ts.factory.createIdentifier("gauntlet"),
                                                ts.factory.createIdentifier(component.internalName)
                                            ),
                                            undefined,
                                            ts.factory.createJsxAttributes(properties)
                                        )
                                    )
                                ],
                                true
                            )
                        )
                    )],
                    ts.NodeFlags.Const
                )
            ),
            ...memberAssignments
        ]
    });

    return ts.factory.createSourceFile(
        [...imports, ...internalDeclarations, ...publicDeclarations, ...components],
        ts.factory.createToken(ts.SyntaxKind.EndOfFileToken),
        ts.NodeFlags.None
    )
}

function makePropertyTypes(component: StandardComponent): ts.TypeElement[] {
    const props = component.props.map(property => {
        return ts.factory.createPropertySignature(
            undefined,
            ts.factory.createIdentifier(property.name),
            !property.optional ? undefined : ts.factory.createToken(ts.SyntaxKind.QuestionToken),
            makeType(property.type)
        )
    });

    if (component.children.type != "none") {
        props.unshift(ts.factory.createPropertySignature(
            undefined,
            ts.factory.createIdentifier("children"),
            ts.factory.createToken(ts.SyntaxKind.QuestionToken),
            makeChildrenType(component.children)
        ))
    }

    return props
}


function makeChildrenType(type: Children): ts.TypeNode {
    switch (type.type) {
        case "members": {
            return ts.factory.createTypeReferenceNode(
                ts.factory.createIdentifier("ElementComponent"),
                [
                    ts.factory.createUnionTypeNode(
                        type.members.map(value => (
                            ts.factory.createTypeQueryNode(
                                ts.factory.createIdentifier(value.componentName),
                                undefined
                            )
                        ))
                    )
                ]
            )
        }
        case "string_or_members": {
            return ts.factory.createTypeReferenceNode(
                ts.factory.createIdentifier("StringOrElementComponent"),
                [
                    ts.factory.createUnionTypeNode(
                        type.members.map(value => (
                            ts.factory.createTypeQueryNode(
                                ts.factory.createIdentifier(value.componentName),
                                undefined
                            )
                        ))
                    )
                ]
            )
        }
        case "string": {
            return ts.factory.createTypeReferenceNode(
                ts.factory.createIdentifier("StringComponent"),
                undefined
            )
        }
        case "none": {
            throw new Error("Cannot construct none children")
        }
    }
}


function makeType(type: PropertyType): ts.TypeNode {
    switch (type.type) {
        case "array": {
            return ts.factory.createArrayTypeNode(makeType(type.nested))
        }
        case "boolean": {
            return ts.factory.createKeywordTypeNode(ts.SyntaxKind.BooleanKeyword)
        }
        case "number": {
            return ts.factory.createKeywordTypeNode(ts.SyntaxKind.NumberKeyword)
        }
        case "string": {
            return ts.factory.createKeywordTypeNode(ts.SyntaxKind.StringKeyword)
        }
        case "function": {
            const params = type.arguments.map(arg => {
                return ts.factory.createParameterDeclaration(
                    undefined,
                    undefined,
                    ts.factory.createIdentifier(arg.name),
                    undefined,
                    !arg.optional ? makeType(arg.type) : ts.factory.createUnionTypeNode([makeType(arg.type), ts.factory.createKeywordTypeNode(ts.SyntaxKind.UndefinedKeyword)]),
                    undefined
                )
            });

            return ts.factory.createFunctionTypeNode(
                undefined,
                params,
                ts.factory.createKeywordTypeNode(ts.SyntaxKind.VoidKeyword)
            )
        }
        default: {
            throw new Error(`unsupported type ${JSON.stringify(type)}`)
        }
    }

}

const genDir = "./gen";
if (!existsSync(genDir)) {
    mkdirSync(genDir);
}

generate("./component_model.json", `${genDir}/components.tsx`)