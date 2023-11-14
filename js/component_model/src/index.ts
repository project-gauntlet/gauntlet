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
type Type = TypeString | TypeNumber | TypeBoolean | TypeReactNode | TypeArray | TypeOr | TypeFunction

type TypeString = {
    name: "string"
}
type TypeNumber = {
    name: "number"
}
type TypeBoolean = {
    name: "boolean"
}
type TypeReactNode = {
    name: "reactnode"
}
type TypeArray = {
    name: "array"
    nested: Type
}
type TypeOr = {
    name: "or"
    nested: Type[]
}
type TypeFunction = {
    name: "function"
}

function generate(componentModelPath: string, outFile: string) {
    const content = readFileSync(componentModelPath).toString();
    const model = JSON.parse(content) as Component[]

    const resultFile = ts.createSourceFile("unused", "", ts.ScriptTarget.Latest, false, ts.ScriptKind.TSX);
    const printer = ts.createPrinter({ newLine: ts.NewLineKind.LineFeed });

    const result = printer.printNode(ts.EmitHint.Unspecified, makeComponents(model), resultFile);

    writeFileSync(outFile, result)
}

function makeComponents(model: Component[]): ts.SourceFile {
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
                        ts.factory.createIdentifier("ReactNode")
                    )
                ])
            ),
            ts.factory.createStringLiteral("react"),
            undefined
        )
    ];

    // ts.factory.createJSDocComment("@internal"), // TODO

    const declaration =
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
                            ts.factory.createIdentifier(`placeholdername__${component.internalName}`),
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
        );

    // abuse the fact that there is no space between multiline comment and content
    // is three a better way to add @internal tag to 'declare global'?
    ts.addSyntheticLeadingComment(declaration, ts.SyntaxKind.MultiLineCommentTrivia, "*@internal", true)

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
                                            ts.factory.createIdentifier(`placeholdername__${component.internalName}`),
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
        [...imports, declaration, ...components],
        ts.factory.createToken(ts.SyntaxKind.EndOfFileToken),
        ts.NodeFlags.None
    )
}

function makeType(type: Type): ts.TypeNode {
    switch (type.name) {
        case "reactnode": {
            return ts.factory.createTypeReferenceNode("ReactNode")
        }
        case "array": {
            return ts.factory.createArrayTypeNode(makeType(type.nested))
        }
        case "or": {
            return ts.factory.createUnionTypeNode(
                type.nested.map(value => makeType(value))
            )
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