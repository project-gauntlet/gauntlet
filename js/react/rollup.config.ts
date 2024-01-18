import commonjs from '@rollup/plugin-commonjs';
import replace from "@rollup/plugin-replace";
import { defineConfig, RollupOptions } from "rollup";


const fixedDevExports = `
var Children = react_developmentExports.Children
var Component = react_developmentExports.Component
var Fragment = react_developmentExports.Fragment
var Profiler = react_developmentExports.Profiler
var PureComponent = react_developmentExports.PureComponent
var StrictMode = react_developmentExports.StrictMode
var Suspense = react_developmentExports.Suspense
var __SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED = react_developmentExports.__SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED;
var cloneElement = react_developmentExports.cloneElement
var createContext = react_developmentExports.createContext
var createElement = react_developmentExports.createElement
var createFactory = react_developmentExports.createFactory
var createRef = react_developmentExports.createRef
var forwardRef = react_developmentExports.forwardRef
var isValidElement = react_developmentExports.isValidElement
var lazy = react_developmentExports.lazy
var memo = react_developmentExports.memo
var startTransition = react_developmentExports.startTransition
var unstable_act = react_developmentExports.unstable_act
var useCallback = react_developmentExports.useCallback
var useContext = react_developmentExports.useContext
var useDebugValue = react_developmentExports.useDebugValue
var useDeferredValue = react_developmentExports.useDeferredValue
var useEffect = react_developmentExports.useEffect
var useId = react_developmentExports.useId
var useImperativeHandle = react_developmentExports.useImperativeHandle
var useInsertionEffect = react_developmentExports.useInsertionEffect
var useLayoutEffect = react_developmentExports.useLayoutEffect
var useMemo = react_developmentExports.useMemo
var useReducer = react_developmentExports.useReducer
var useRef = react_developmentExports.useRef
var useState = react_developmentExports.useState
var useSyncExternalStore = react_developmentExports.useSyncExternalStore
var useTransition = react_developmentExports.useTransition
var version = react_developmentExports.version

export { Children, Component, Fragment, Profiler, PureComponent, StrictMode, Suspense, __SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED, cloneElement, createContext, createElement, createFactory, createRef, react_development as default, forwardRef, isValidElement, lazy, memo, startTransition, unstable_act, useCallback, useContext, useDebugValue, useDeferredValue, useEffect, useId, useImperativeHandle, useInsertionEffect, useLayoutEffect, useMemo, useReducer, useRef, useState, useSyncExternalStore, useTransition, version };
`;

const config = (nodeEnv: string, reactBundle: string, outDir: string): RollupOptions => {

    return {
        input: [
            `../../node_modules/react/cjs/react.${reactBundle}`,
            `../../node_modules/react/cjs/react-jsx-runtime.${reactBundle}`,
        ],
        output: {
            dir: outDir,
            format: 'esm',
        },
        external: ['react'],
        plugins: [
            commonjs(),
            replace({
                delimiters: ['', ''],
                values: {
                    // npm bundle of React has references to npm process
                    'process.env.NODE_ENV': JSON.stringify(nodeEnv),
                    // To fix exports in development bundle https://github.com/rollup/plugins/issues/1546
                    'export { react_development as default };': fixedDevExports,
                }
            })
        ]
    }
}

// 'node_modules/react/cjs/react.development.js',
// 'node_modules/react/cjs/react.production.min.js',
// 'node_modules/react/cjs/react-jsx-dev-runtime.development.js,' # essentially same as react-jsx-runtime.development.js but jsx -> jsxDev, and no jsxs
// 'node_modules/react/cjs/react-jsx-dev-runtime.production.min.js', # empty
// 'node_modules/react/cjs/react-jsx-dev-runtime.profiling.min.js', # empty
// 'node_modules/react/cjs/react-jsx-runtime.development.js',
// 'node_modules/react/cjs/react-jsx-runtime.production.min.js',
// 'node_modules/react/cjs/react-jsx-runtime.profiling.min.js', # same as react-jsx-runtime.production.min.js

export default defineConfig([
    config('production', 'production.min.js', 'dist/prod'),
    config('development', 'development.js', 'dist/dev')
])
