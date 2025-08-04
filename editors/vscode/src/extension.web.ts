import * as vscode from 'vscode';
import { ExtensionContext, window } from "vscode";
import { loadTinymistConfig } from "./config";
import { tinymistActivate, tinymistDeactivate, FeatureEntry } from "./extension.shared";
import { extensionState } from "./state";
import { LanguageClient } from "vscode-languageclient/browser";
import { LanguageState } from "./lsp";

// Import the WASM implementation when available
// Currently commented out since we're still developing the integration
// import { createMonacoLanguageClient } from "../../crates/tinymist-wasm";
// import TinymistWorker from "../../crates/tinymist-wasm/worker?worker";

// Register the Web LSP features
async function webLspActivate(context: { context: ExtensionContext }) {
  // This is where we'll initialize the WASM-based language server
  window.showInformationMessage("Tinymist Web LSP support is being implemented");

  // When the WASM implementation is fully ready, we would do:
  // 1. Create a worker for the language server
  // const worker = new TinymistWorker();
  // 2. Create a language client that connects to the worker
  // const languageClient = createMonacoLanguageClient(worker);
  // 3. Start the language client
  // await languageClient.start();
}

// Create a version of the activate table for web
const webActivateTable = (): FeatureEntry[] => [
  [true, webLspActivate],
];

export async function activate(context: ExtensionContext): Promise<void> {
  // Configure for web environment
  extensionState.features = {
    web: true,
    lsp: true,  // We're enabling LSP for web now
    export: false,
    task: false,
    wordSeparator: true,
    label: false,
    package: false,
    tool: false,
    devKit: false,
    dragAndDrop: false,
    copyAndPaste: false,
    onEnter: false,
    testing: false,
    testingDebug: false,
    preview: false,
    language: true,  // Enable language features
    renderDocs: false,
  };

  // Set up the web language client
  LanguageState.Client = LanguageClient;

  try {
    return await tinymistActivate(context, {
      activateTable: webActivateTable,
      config: loadTinymistConfig(),
    });
  } catch (e) {
    void window.showErrorMessage(`Failed to activate tinymist in web mode: ${e}`);
    throw e;
  }
}

export async function deactivate(): Promise<void> {
  tinymistDeactivate({
    activateTable: webActivateTable,
  });
}
