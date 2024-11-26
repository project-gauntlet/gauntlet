import ts, {
    ExpressionStatement,
    ImportDeclaration,
    isExportDeclaration,
    isExportSpecifier,
    isIdentifier,
    isNamedExports,
    ScriptKind,
    Statement
} from "typescript";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";


function generate(outFile: string, sourceFile: ts.SourceFile) {
    const resultFile = ts.createSourceFile("unused", "", ts.ScriptTarget.Latest, false, ts.ScriptKind.JS);
    const printer = ts.createPrinter({ newLine: ts.NewLineKind.LineFeed });

    const result = printer.printNode(ts.EmitHint.Unspecified, sourceFile, resultFile);

    writeFileSync(outFile, result)
}

function collectExports(inputFile: string): string[] {
    const sourceFile = ts.createSourceFile(
        "input.js",
        readFileSync(inputFile).toString(),
        ts.ScriptTarget.ESNext,
        /*setParentNodes */ false,
        ScriptKind.JS
    );

    const result: string[] = []

    for (const statement of sourceFile.statements) {
        if (isExportDeclaration(statement)) {
            const exportClause = statement.exportClause;
            if (exportClause) {
                if (isNamedExports(exportClause)) {
                    for (const element of exportClause.elements) {
                        if (isExportSpecifier(element)) {
                            if (isIdentifier(element.name)) {
                                if (typeof element.name.escapedText === "string") {
                                    if (element.name.escapedText.startsWith("___")) {
                                        result.push(element.name.escapedText.slice(1)) // remove one _, typescript special case
                                    } else {
                                        result.push(element.name.escapedText)
                                    }
                                } else {
                                    throw new Error(`unexpected export clause element element name type: ${JSON.stringify(element)}`)
                                }
                            } else {
                                throw new Error(`unknown export clause element element name type: ${JSON.stringify(element)}`)
                            }
                        } else {
                            throw new Error(`unknown export clause element: ${JSON.stringify(element)}`)
                        }
                    }
                } else {
                    throw new Error(`unknown export clause: ${JSON.stringify(exportClause)}`)
                }
            }
        }
    }

    return result
}


function generateInternal(exportConfig: Record<string, { importUrl: string, exports: string[] }>): ts.SourceFile {

    function createImport(exports: string[], namespace:string, importString: string): ImportDeclaration {
        return ts.factory.createImportDeclaration(
            undefined,
            ts.factory.createImportClause(
                false,
                undefined,
                ts.factory.createNamedImports(exports.map(value => {
                    return ts.factory.createImportSpecifier(
                        false,
                        ts.factory.createIdentifier(value),
                        ts.factory.createIdentifier(`${namespace}_${value}`)
                    )
                }))
            ),
            ts.factory.createStringLiteral(importString),
            undefined
        )
    }

    const initialDeclarations: Statement[] = Object.entries(exportConfig)
        .map(([namespace, { importUrl, exports }]) => createImport(exports, namespace, importUrl));

    function createAssignment(namespace: string, variableName: string): ExpressionStatement {
        return ts.factory.createExpressionStatement(ts.factory.createBinaryExpression(
            ts.factory.createPropertyAccessExpression(
                ts.factory.createIdentifier("globalThis"),
                ts.factory.createIdentifier(`${namespace}_${variableName}`)
            ),
            ts.factory.createToken(ts.SyntaxKind.EqualsToken),
            ts.factory.createIdentifier(`${namespace}_${variableName}`)
        ))
    }

    const assignments: Statement[] = Object.entries(exportConfig)
        .flatMap(([namespace, { exports }]) => exports.map(value => createAssignment(namespace, value)));

    return ts.factory.createSourceFile(
        [
            ...initialDeclarations,
            ...assignments,
        ],
        ts.factory.createToken(ts.SyntaxKind.EndOfFileToken),
        ts.NodeFlags.None
    )
}

function generateExternal(namespace: string, exports: string[]): ts.SourceFile {

    const assignments = exports.map(value => {

        return ts.factory.createVariableStatement(
            undefined,
            ts.factory.createVariableDeclarationList(
                [ts.factory.createVariableDeclaration(
                    ts.factory.createIdentifier(`${namespace}_${value}`),
                    undefined,
                    undefined,
                    ts.factory.createPropertyAccessExpression(
                        ts.factory.createIdentifier("globalThis"),
                        ts.factory.createIdentifier(`${namespace}_${value}`)
                    )
                )],
                ts.NodeFlags.Const
            )
        );
    });

    const exportDeclaration = ts.factory.createExportDeclaration(
        undefined,
        false,
        ts.factory.createNamedExports(exports.map(value => {
            return ts.factory.createExportSpecifier(
                false,
                ts.factory.createIdentifier(`${namespace}_${value}`),
                ts.factory.createIdentifier(value)
            )
        })),
        undefined,
        undefined
    );

    return ts.factory.createSourceFile(
        [...assignments, exportDeclaration],
        ts.factory.createToken(ts.SyntaxKind.EndOfFileToken),
        ts.NodeFlags.None
    )
}

const outDir = `./dist`;

if (!existsSync(outDir)) {
    mkdirSync(outDir);
}

const componentExports = collectExports(`../api/dist/gen/components.js`);
const helpersExports = collectExports(`../api/dist/helpers.js`);
const hooksExports = collectExports(`../api/dist/hooks.js`);
const coreExports = collectExports(`../core/dist/core.js`);

// prod bundle exports are identical and hopefully it stays like this in future
const reactExports = collectExports(`../react/dist/dev/react.development.js`);
const reactJsxRuntimeExports = collectExports(`../react/dist/dev/react-jsx-runtime.development.js`);

const internalAllExports = collectExports(`../core/dist/internal-all.js`);
const internalLinuxExports = collectExports(`../core/dist/internal-linux.js`);
const internalMacosExports = collectExports(`../core/dist/internal-macos.js`);

generate(
    `${outDir}/bridge-bootstrap.js`,
    generateInternal({
        "GauntletComponents": { importUrl: "ext:gauntlet/api/components.js", exports: componentExports },
        "GauntletHelpers": { importUrl: "ext:gauntlet/api/helpers.js", exports: helpersExports },
        "GauntletHooks": { importUrl: "ext:gauntlet/api/hooks.js", exports: hooksExports },
        "GauntletCore": { importUrl: "ext:gauntlet/core.js", exports: coreExports },
        "GauntletReact": { importUrl: "ext:gauntlet/react.js", exports: reactExports },
        "GauntletReactJsxRuntime": { importUrl: "ext:gauntlet/react-jsx-runtime.js", exports: reactJsxRuntimeExports },
    })
)

generate(`${outDir}/bridge-internal-all-bootstrap.js`, generateInternal({
    "GauntletInternalAll": { importUrl: "ext:gauntlet/internal-all.js", exports: internalAllExports }
}))
generate(`${outDir}/bridge-internal-linux-bootstrap.js`, generateInternal({
    "GauntletInternalLinux": { importUrl: "ext:gauntlet/internal-linux.js", exports: internalLinuxExports }
}))
generate(`${outDir}/bridge-internal-macos-bootstrap.js`, generateInternal({
    "GauntletInternalMacos": { importUrl: "ext:gauntlet/internal-macos.js", exports: internalMacosExports }
}))


generate(`${outDir}/bridge-components.js`, generateExternal("GauntletComponents", componentExports))
generate(`${outDir}/bridge-helpers.js`, generateExternal("GauntletHelpers", helpersExports))
generate(`${outDir}/bridge-hooks.js`, generateExternal("GauntletHooks", hooksExports))
generate(`${outDir}/bridge-core.js`, generateExternal("GauntletCore", coreExports))
generate(`${outDir}/bridge-react.js`, generateExternal("GauntletReact", reactExports))
generate(`${outDir}/bridge-react-jsx-runtime.js`, generateExternal("GauntletReactJsxRuntime", reactJsxRuntimeExports))

generate(`${outDir}/bridge-internal-all.js`, generateExternal("GauntletInternalAll", internalAllExports))
generate(`${outDir}/bridge-internal-linux.js`, generateExternal("GauntletInternalLinux", internalLinuxExports))
generate(`${outDir}/bridge-internal-macos.js`, generateExternal("GauntletInternalMacos", internalMacosExports))


