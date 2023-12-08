import ts from "typescript";
import { readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";

type Component = {
    internalName: string,
    name: string,
    props: Property[],
    members: Record<string, string>,
}

type Property = {
    name: string
    optional: boolean
    type: Type
}
type Type = TypeString | TypeNumber | TypeBoolean | TypeStringComponent | TypeComponent | TypeArray | TypeFunction

type TypeString = {
    name: "string"
}
type TypeNumber = {
    name: "number"
}
type TypeBoolean = {
    name: "boolean"
}
type TypeComponent = {
    name: "components"
    components: string[]
}
type TypeStringComponent = {
    name: "stringcomponent"
}
type TypeArray = {
    name: "array"
    nested: Type
}
type TypeFunction = {
    name: "function"
}

// TODO freeze objects

function generate(componentModelPath: string, outFile: string) {
    const content = readFileSync(componentModelPath).toString();
    const model = JSON.parse(content) as Component[]

    const resultFile = ts.createSourceFile("unused", "", ts.ScriptTarget.Latest, false, ts.ScriptKind.TSX);
    const printer = ts.createPrinter({ newLine: ts.NewLineKind.LineFeed });

    const result = printer.printNode(ts.EmitHint.Unspecified, makeComponents(model), resultFile);

    writeFileSync(outFile, result)
}

function makeComponents(modelInput: Component[]): ts.SourceFile {
    const model = modelInput.filter(component => component.internalName !== "container");

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
            ts.factory.createIdentifier("Component"),
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
                        ts.factory.createIdentifier("Component"),
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
        )
    ];



    const internalDeclarations = [
        ts.factory.createVariableStatement(
            undefined,
            ts.factory.createVariableDeclarationList(
                [ts.factory.createVariableDeclaration(
                    ts.factory.createIdentifier("internalType"),
                    undefined,
                    undefined,
                    ts.factory.createCallExpression(
                        ts.factory.createIdentifier("Symbol"),
                        undefined,
                        [ts.factory.createStringLiteral("GAUNTLET:INTERNAL_TYPE")]
                    )
                )],
                ts.NodeFlags.Const
            )
        ),
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
                            ts.factory.createTypeLiteralNode(component.props.map(property => {
                                return ts.factory.createPropertySignature(
                                    undefined,
                                    ts.factory.createIdentifier(property.name),
                                    !property.optional ? undefined : ts.factory.createToken(ts.SyntaxKind.QuestionToken),
                                    makeType(property.type)
                                )
                            }))
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

        const componentFCType = ts.factory.createTypeReferenceNode(
            ts.factory.createIdentifier("FC"),
            [ts.factory.createTypeReferenceNode(
                ts.factory.createIdentifier(`${component.name}Props`),
                undefined
            )]
        )

        const componentType = Object.entries(component.members).length == 0
            ? componentFCType
            : ts.factory.createIntersectionTypeNode([
                componentFCType,
                ts.factory.createTypeLiteralNode(
                    Object.entries(component.members).map(([key, value]) => {
                        return ts.factory.createPropertySignature(
                            undefined,
                            ts.factory.createIdentifier(key),
                            undefined,
                            ts.factory.createTypeQueryNode(
                                ts.factory.createIdentifier(value),
                                undefined
                            )
                        );
                    })
                )
            ])

        return [
            ts.factory.createInterfaceDeclaration(
                [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
                ts.factory.createIdentifier(`${component.name}Props`),
                undefined,
                undefined,
                component.props.map(property => {
                    return ts.factory.createPropertySignature(
                        undefined,
                        ts.factory.createIdentifier(property.name),
                        !property.optional ? undefined : ts.factory.createToken(ts.SyntaxKind.QuestionToken),
                        makeType(property.type)
                    );
                })
            ),
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
                            [ts.factory.createParameterDeclaration(
                                undefined,
                                undefined,
                                ts.factory.createIdentifier("props"),
                                undefined,
                                ts.factory.createTypeReferenceNode(
                                    ts.factory.createIdentifier(`${component.name}Props`),
                                    undefined
                                ),
                                undefined
                            )],
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
                                            ts.factory.createJsxAttributes(component.props.map((prop) => (
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
                                            )))
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
            ts.factory.createExpressionStatement(ts.factory.createBinaryExpression(
                ts.factory.createElementAccessExpression(
                    ts.factory.createParenthesizedExpression(ts.factory.createAsExpression(
                        ts.factory.createIdentifier(component.name),
                        ts.factory.createKeywordTypeNode(ts.SyntaxKind.AnyKeyword)
                    )),
                    ts.factory.createIdentifier("internalType")
                ),
                ts.factory.createToken(ts.SyntaxKind.EqualsToken),
                ts.factory.createStringLiteral(component.internalName)
            )),
            ...Object.entries(component.members).map(([key, value]) => {
                return ts.factory.createExpressionStatement(ts.factory.createBinaryExpression(
                    ts.factory.createPropertyAccessExpression(
                        ts.factory.createIdentifier(component.name),
                        ts.factory.createIdentifier(key)
                    ),
                    ts.factory.createToken(ts.SyntaxKind.EqualsToken),
                    ts.factory.createIdentifier(value)
                ))
            })
        ]
    });

    return ts.factory.createSourceFile(
        [...imports, ...internalDeclarations, ...publicDeclarations, ...components],
        ts.factory.createToken(ts.SyntaxKind.EndOfFileToken),
        ts.NodeFlags.None
    )
}

function makeType(type: Type): ts.TypeNode {
    switch (type.name) {
        case "components": {
            return ts.factory.createTypeReferenceNode(
                ts.factory.createIdentifier("Component"),
                [
                    ts.factory.createUnionTypeNode(
                        type.components.map(value => (
                            ts.factory.createTypeQueryNode(
                                ts.factory.createIdentifier(value),
                                undefined
                            )
                        ))
                    )
                ]
            )
        }
        case "stringcomponent": {
            return ts.factory.createTypeReferenceNode(
                ts.factory.createIdentifier("StringComponent"),
                undefined
            )
        }
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
            return ts.factory.createFunctionTypeNode(
                undefined,
                [],
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