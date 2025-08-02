import {
    createConnection,
    BrowserMessageReader,
    BrowserMessageWriter,
    TextDocuments,
    Diagnostic,
    DiagnosticSeverity,
    CompletionItem,
    CompletionItemKind,
    TextDocumentSyncKind,
    InitializeResult
} from 'vscode-languageserver/browser';

import { TextDocument } from 'vscode-languageserver-textdocument';
import init, { TinymistWasmLanguageServer } from "./pkg";

await init();
const server = new TinymistWasmLanguageServer(); 

console.log('Language server worker running...');

const reader = new BrowserMessageReader(self);
const writer = new BrowserMessageWriter(self);
const connection = createConnection(reader, writer);

const documents = new TextDocuments(TextDocument);

connection.onInitialize((params) => {
    const capabilities = server.initialize(params);
    return { capabilities };
});

documents.onDidChangeContent(change => {
    server.did_change({
        textDocument: {
            uri: change.document.uri,
            version: change.document.version
        },
        contentChanges: [{
            text: change.document.getText()
        }]
    });
});

connection.onCompletion((_textDocumentPosition) => {
    return server.completion(_textDocumentPosition);
});

documents.listen(connection);
connection.listen();
