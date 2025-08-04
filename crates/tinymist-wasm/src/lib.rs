//! Tinymist WASM language server implementation.
//!
//! This crate provides a WebAssembly-compatible implementation of the Tinymist
//! language server for use with Monaco Editor in the browser.
#![warn(missing_docs)]

use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use js_sys::{Array, Object};
use lsp_types::{DocumentSymbol, DocumentSymbolResponse};

/// Initialize panic hook for better error messages in the browser console
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

/// TinymistLanguageServer implements the LSP protocol for Typst documents
/// in a WebAssembly environment
#[wasm_bindgen]
pub struct TinymistLanguageServer {
    version: String,
    /// Store document contents by URI
    documents: HashMap<String, String>,
}

#[wasm_bindgen]
impl TinymistLanguageServer {
    /// Create a new language server.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            documents: HashMap::new(),
        }
    }
    
    /// Get the version of the language server.
    pub fn version(&self) -> String {
        self.version.clone()
    }
    
    /// Get a greeting message.
    pub fn greet(&self) -> String {
        format!("Hello from Tinymist WASM v{}!", self.version)
    }
    
    /// Update or add a document in the language server's storage
    pub fn update_document(&mut self, uri: String, content: String) {
        self.documents.insert(uri.clone(), content);
        web_sys::console::log_1(&format!("Document updated: {}", uri).into());
    }
    
    /// Remove a document from the language server's storage
    pub fn remove_document(&mut self, uri: String) {
        self.documents.remove(&uri);
        web_sys::console::log_1(&format!("Document removed: {}", uri).into());
    }
    
    // LSP feature implementations

    /// Get completion items for the specified position.
    pub fn get_completions(&self, uri: String, line: u32, character: u32) -> JsValue {
        // Mock implementation - in the future will use tinymist-core
        let completions = Array::new();
        
        // Add some basic completions
        let item1 = Object::new();
        js_sys::Reflect::set(&item1, &"label".into(), &"#set".into()).unwrap();
        js_sys::Reflect::set(&item1, &"kind".into(), &14.into()).unwrap(); // Keyword
        js_sys::Reflect::set(&item1, &"detail".into(), &"Set a style property".into()).unwrap();
        completions.push(&item1);
        
        let item2 = Object::new();
        js_sys::Reflect::set(&item2, &"label".into(), &"#show".into()).unwrap();
        js_sys::Reflect::set(&item2, &"kind".into(), &14.into()).unwrap(); // Keyword
        js_sys::Reflect::set(&item2, &"detail".into(), &"Define a style rule".into()).unwrap();
        completions.push(&item2);
        
        let item3 = Object::new();
        js_sys::Reflect::set(&item3, &"label".into(), &"text".into()).unwrap();
        js_sys::Reflect::set(&item3, &"kind".into(), &3.into()).unwrap(); // Function
        js_sys::Reflect::set(&item3, &"detail".into(), &"Create text content".into()).unwrap();
        completions.push(&item3);
        
        completions.into()
    }
    
    /// Get hover information for the specified position.
    pub fn get_hover(&self, uri: String, line: u32, character: u32) -> JsValue {
        // Check if we have the document
        if !self.documents.contains_key(&uri) {
            return JsValue::NULL;
        }
        
        let hover = Object::new();
        let contents = Object::new();
        
        js_sys::Reflect::set(&contents, &"kind".into(), &"markdown".into()).unwrap();
        js_sys::Reflect::set(&contents, &"value".into(), &"**Typst Element**\n\nTypst language element at this position.".into()).unwrap();
        js_sys::Reflect::set(&hover, &"contents".into(), &contents).unwrap();
        
        hover.into()
    }
    
    /// Get document symbols for the specified document
    pub fn get_document_symbols(&self, uri: String) -> JsValue {
        if !self.documents.contains_key(&uri) {
            return Array::new().into();
        }

        let content = &self.documents[&uri];
        
        // Parse the typst source and extract symbols
        let source = typst::syntax::Source::detached(content);
        
        // Use tinymist-query's public DocumentSymbolRequest API
        use tinymist_query::{DocumentSymbolRequest, SyntaxRequest, PositionEncoding};
        
        let path = std::path::PathBuf::from(&uri);
        let request = DocumentSymbolRequest { path };
        
        if let Some(DocumentSymbolResponse::Nested(symbols)) = request.request(&source, PositionEncoding::Utf16) {
            let js_symbols = Array::new();
            
            for symbol in symbols {
                if let Some(symbol_obj) = self.document_symbol_to_js(&symbol) {
                    js_symbols.push(&symbol_obj);
                }
            }
            
            js_symbols.into()
        } else {
            Array::new().into()
        }
    }
    
    /// Go to definition at the specified position
    pub fn goto_definition(&self, uri: String, line: u32, character: u32) -> JsValue {
        todo!("Implement goto_definition")
    }
    
    /// Go to declaration at the specified position
    pub fn goto_declaration(&self, uri: String, line: u32, character: u32) -> JsValue {
        todo!("Implement goto_declaration")
    }
    
    /// Find references at the specified position
    pub fn find_references(&self, uri: String, line: u32, character: u32) -> JsValue {
        todo!("Implement find_references")
    }
    
    /// Get folding ranges for the document
    pub fn folding_range(&self, uri: String) -> JsValue {
        todo!("Implement folding_range")
    }
    
    /// Get selection range at the specified positions
    pub fn selection_range(&self, uri: String, positions: JsValue) -> JsValue {
        todo!("Implement selection_range")
    }
    
    /// Get document highlights at the specified position
    pub fn document_highlight(&self, uri: String, line: u32, character: u32) -> JsValue {
        todo!("Implement document_highlight")
    }
    
    /// Get semantic tokens for the full document
    pub fn semantic_tokens_full(&self, uri: String) -> JsValue {
        todo!("Implement semantic_tokens_full")
    }
    
    /// Get semantic tokens delta for the document
    pub fn semantic_tokens_delta(&self, uri: String, previous_result_id: String) -> JsValue {
        todo!("Implement semantic_tokens_delta")
    }
    
    /// Format the document
    pub fn formatting(&self, uri: String) -> JsValue {
        todo!("Implement formatting")
    }
    
    /// Get inlay hints for the document in the specified range
    pub fn inlay_hint(&self, uri: String, start_line: u32, start_char: u32, end_line: u32, end_char: u32) -> JsValue {
        todo!("Implement inlay_hint")
    }
    
    /// Get document colors
    pub fn document_color(&self, uri: String) -> JsValue {
        todo!("Implement document_color")
    }
    
    /// Get document links
    pub fn document_link(&self, uri: String) -> JsValue {
        todo!("Implement document_link")
    }
    
    /// Get color presentation for a specific color at the specified range
    pub fn color_presentation(&self, uri: String, color: JsValue, start_line: u32, start_char: u32, end_line: u32, end_char: u32) -> JsValue {
        todo!("Implement color_presentation")
    }
    
    /// Get code actions for the specified range
    pub fn code_action(&self, uri: String, start_line: u32, start_char: u32, end_line: u32, end_char: u32, context: JsValue) -> JsValue {
        todo!("Implement code_action")
    }
    
    /// Get code lenses for the document
    pub fn code_lens(&self, uri: String) -> JsValue {
        todo!("Implement code_lens")
    }
    
    /// Get signature help at the specified position
    pub fn signature_help(&self, uri: String, line: u32, character: u32) -> JsValue {
        todo!("Implement signature_help")
    }
    
    /// Rename the symbol at the specified position
    pub fn rename(&self, uri: String, line: u32, character: u32, new_name: String) -> JsValue {
        todo!("Implement rename")
    }
    
    /// Prepare for rename at the specified position
    pub fn prepare_rename(&self, uri: String, line: u32, character: u32) -> JsValue {
        todo!("Implement prepare_rename")
    }
    
    /// Get workspace symbols matching the pattern
    pub fn symbol(&self, pattern: String) -> JsValue {
        todo!("Implement symbol")
    }
    
    /// Handle on_enter events
    pub fn on_enter(&self, uri: String, start_line: u32, start_char: u32, end_line: u32, end_char: u32) -> JsValue {
        todo!("Implement on_enter")
    }
    
    /// Handle will_rename_files events
    pub fn will_rename_files(&self, file_renames: JsValue) -> JsValue {
        todo!("Implement will_rename_files")
    }
    
    // Helper methods
    
    /// Convert a DocumentSymbol to a JavaScript object
    fn document_symbol_to_js(&self, symbol: &DocumentSymbol) -> Option<Object> {
        let js_symbol = Object::new();
        
        // Set the name
        js_sys::Reflect::set(&js_symbol, &"name".into(), &symbol.name.clone().into()).ok()?;
        
        // Set the kind (using LSP SymbolKind numbers)
        let kind_num: u32 = match symbol.kind {
            lsp_types::SymbolKind::FILE => 1,
            lsp_types::SymbolKind::MODULE => 2,
            lsp_types::SymbolKind::NAMESPACE => 3,
            lsp_types::SymbolKind::PACKAGE => 4,
            lsp_types::SymbolKind::CLASS => 5,
            lsp_types::SymbolKind::METHOD => 6,
            lsp_types::SymbolKind::PROPERTY => 7,
            lsp_types::SymbolKind::FIELD => 8,
            lsp_types::SymbolKind::CONSTRUCTOR => 9,
            lsp_types::SymbolKind::ENUM => 10,
            lsp_types::SymbolKind::INTERFACE => 11,
            lsp_types::SymbolKind::FUNCTION => 12,
            lsp_types::SymbolKind::VARIABLE => 13,
            lsp_types::SymbolKind::CONSTANT => 14,
            lsp_types::SymbolKind::STRING => 15,
            lsp_types::SymbolKind::NUMBER => 16,
            lsp_types::SymbolKind::BOOLEAN => 17,
            lsp_types::SymbolKind::ARRAY => 18,
            lsp_types::SymbolKind::OBJECT => 19,
            lsp_types::SymbolKind::KEY => 20,
            lsp_types::SymbolKind::NULL => 21,
            lsp_types::SymbolKind::ENUM_MEMBER => 22,
            lsp_types::SymbolKind::STRUCT => 23,
            lsp_types::SymbolKind::EVENT => 24,
            lsp_types::SymbolKind::OPERATOR => 25,
            lsp_types::SymbolKind::TYPE_PARAMETER => 26,
            _ => 13, // Default to VARIABLE for unknown kinds
        };
        js_sys::Reflect::set(&js_symbol, &"kind".into(), &kind_num.into()).ok()?;
        
        // Set the range
        let range_obj = Object::new();
        let start_obj = Object::new();
        let end_obj = Object::new();
        
        js_sys::Reflect::set(&start_obj, &"line".into(), &symbol.range.start.line.into()).ok()?;
        js_sys::Reflect::set(&start_obj, &"character".into(), &symbol.range.start.character.into()).ok()?;
        js_sys::Reflect::set(&end_obj, &"line".into(), &symbol.range.end.line.into()).ok()?;
        js_sys::Reflect::set(&end_obj, &"character".into(), &symbol.range.end.character.into()).ok()?;
        
        js_sys::Reflect::set(&range_obj, &"start".into(), &start_obj).ok()?;
        js_sys::Reflect::set(&range_obj, &"end".into(), &end_obj).ok()?;
        
        js_sys::Reflect::set(&js_symbol, &"range".into(), &range_obj).ok()?;
        js_sys::Reflect::set(&js_symbol, &"selectionRange".into(), &range_obj).ok()?;
        
        // Set detail if available
        if let Some(detail) = &symbol.detail {
            js_sys::Reflect::set(&js_symbol, &"detail".into(), &detail.clone().into()).ok()?;
        }
        
        // Set children if available
        if let Some(children) = &symbol.children {
            let js_children = Array::new();
            for child in children {
                if let Some(child_obj) = self.document_symbol_to_js(child) {
                    js_children.push(&child_obj);
                }
            }
            js_sys::Reflect::set(&js_symbol, &"children".into(), &js_children).ok()?;
        }
        
        Some(js_symbol)
    }
}