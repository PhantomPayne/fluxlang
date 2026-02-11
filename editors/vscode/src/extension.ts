import * as path from 'path';
import * as vscode from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
  // Get the LSP server executable path
  // In production, this would be the compiled flux-lsp binary
  const serverCommand = process.env.FLUX_LSP_PATH || 'flux-lsp';

  const serverOptions: ServerOptions = {
    command: serverCommand,
    args: [],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'flux' }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.flux'),
    },
  };

  client = new LanguageClient(
    'fluxLanguageServer',
    'Flux Language Server',
    serverOptions,
    clientOptions
  );

  client.start();

  vscode.window.showInformationMessage('Flux Language Server activated');
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
