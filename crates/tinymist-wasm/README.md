# tinymist-monaco

This package integrates the Tinymist typst language server with Monaco Editor, providing a rich editing experience for Typst documents in the browser.

## Features

- Syntax highlighting for Typst files
- Autocompletion
- Hover information
- More features coming soon!

## Installation

```bash
npm install tinymist-monaco monaco-editor
```

## Usage

```typescript
import { createMonacoLanguageClient } from 'tinymist-monaco';
import TinymistWorker from 'tinymist-monaco/worker?worker';

// Create a Monaco editor instance
const editor = monaco.editor.create(document.getElementById('editor'), {
    value: 'Hello #strong[world]!',
    language: 'typst'
});

// Create the worker and language client
const worker = new TinymistWorker();
const languageClient = createMonacoLanguageClient(worker);

// Start the language client
languageClient.start();
```

## Development

This package contains:
- A TypeScript wrapper around the Monaco editor integration
- A WebAssembly module with the Tinymist language server

### Building

```bash
# Build the WASM module
wasm-pack build crates/tinymist-wasm

# Build the TypeScript package
npm run build
```

## License

Apache-2.0
