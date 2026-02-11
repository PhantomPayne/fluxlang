use dashmap::DashMap;
use flux_sema::{FileId, SymbolBridge, Vfs};
use std::path::PathBuf;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

pub struct FluxLanguageServer {
    client: Client,
    vfs: Arc<Vfs>,
    symbol_bridge: Arc<SymbolBridge>,
    document_map: DashMap<Url, FileId>,
}

impl FluxLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            vfs: Arc::new(Vfs::new()),
            symbol_bridge: Arc::new(SymbolBridge::new()),
            document_map: DashMap::new(),
        }
    }

    fn analyze_document(&self, file_id: FileId) {
        if let Some(file_data) = self.vfs.get_file(file_id) {
            match flux_syntax::parse(&file_data.content) {
                Ok(ast) => {
                    self.symbol_bridge.analyze_file(file_id, &ast);
                }
                Err(_) => {
                    // Handle parse errors
                }
            }
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for FluxLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "flux-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        ..Default::default()
                    },
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Flux LSP initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;

        let path = PathBuf::from(uri.path());
        let file_id = self.vfs.set_file_content(&path, content);

        self.document_map.insert(uri.clone(), file_id);
        self.analyze_document(file_id);

        self.client
            .log_message(MessageType::INFO, format!("Opened document: {}", uri))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        if let Some(change) = params.content_changes.first() {
            let path = PathBuf::from(uri.path());
            let file_id = self.vfs.set_file_content(&path, change.text.clone());

            self.document_map.insert(uri.clone(), file_id);
            self.analyze_document(file_id);
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.document_map.remove(&uri);

        self.client
            .log_message(MessageType::INFO, format!("Closed document: {}", uri))
            .await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(file_id_ref) = self.document_map.get(&uri) {
            let file_id = *file_id_ref;

            if let Some(file_data) = self.vfs.get_file(file_id) {
                // Convert position to offset
                let offset = position_to_offset(&file_data.content, position);

                if let Some(symbol) = self.symbol_bridge.symbol_at_position(file_id, offset) {
                    let hover_text = format!("**{}**: `{}`", symbol.name, symbol.ty);

                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: hover_text,
                        }),
                        range: None,
                    }));
                }
            }
        }

        Ok(None)
    }
}

fn position_to_offset(content: &str, position: Position) -> usize {
    let mut offset = 0;
    let mut current_line = 0;

    for line in content.lines() {
        if current_line == position.line as usize {
            return offset + position.character as usize;
        }
        offset += line.len() + 1; // +1 for newline
        current_line += 1;
    }

    offset
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| FluxLanguageServer::new(client));

    Server::new(stdin, stdout, socket).serve(service).await;
}
