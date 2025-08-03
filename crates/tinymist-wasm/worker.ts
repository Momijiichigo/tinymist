import {
    createConnection,
    BrowserMessageReader,
    BrowserMessageWriter,
    TextDocuments,
    CompletionItem,
    CompletionItemKind,
    TextDocumentSyncKind,
    InitializeResult
} from 'vscode-languageserver/browser';

import { TextDocument } from 'vscode-languageserver-textdocument';
// We need to import the WASM module
// This will be implemented properly once we have the full WASM build
import * as wasmModule from "./pkg/tinymist_wasm";

// In a real implementation, we would initialize the WASM module
// For now, we'll just create a basic language server

console.log('Language server worker running...');
console.log('Tinymist WASM language server stub loaded');

const reader = new BrowserMessageReader(self as any);
const writer = new BrowserMessageWriter(self as any);
const connection = createConnection(reader, writer);

const documents = new TextDocuments(TextDocument);

// Store document contents
const documentContents = new Map<string, string>();

connection.onInitialize((_params) => {
    // Basic server capabilities
    const capabilities = {
        textDocumentSync: TextDocumentSyncKind.Incremental,
        completionProvider: {
            resolveProvider: false,
            triggerCharacters: ['.', '#', '@']
        },
        hoverProvider: true
    };
    
    return {
        capabilities,
        serverInfo: {
            name: "Tinymist Language Server",
            version: "0.1.0" // This would come from the WASM module
        }
    } as InitializeResult;
});

// Document management
documents.onDidOpen(event => {
    documentContents.set(event.document.uri, event.document.getText());
});

documents.onDidChangeContent(change => {
    documentContents.set(change.document.uri, change.document.getText());
});

documents.onDidClose(event => {
    documentContents.delete(event.document.uri);
});

// Implement basic completion
connection.onCompletion((_textDocumentPosition) => {
    // Simple completion items for now
    return [
        {
            label: '#set',
            kind: CompletionItemKind.Keyword,
            detail: 'Set a style property'
        },
        {
            label: '#show',
            kind: CompletionItemKind.Keyword,
            detail: 'Define a style rule'
        },
        {
            label: 'text',
            kind: CompletionItemKind.Function,
            detail: 'Create text content'
        }
    ];
});

// Implement basic hover
connection.onHover((_params) => {
    return {
        contents: {
            kind: 'markdown',
            value: '**Typst Element**\n\nThis is a placeholder hover text.'
        }
    };
});

// Start the language server
documents.listen(connection);
connection.listen();
