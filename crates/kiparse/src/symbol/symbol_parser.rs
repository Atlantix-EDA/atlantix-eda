use super::types::*;
use crate::error::{KicadError, Result};
use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
enum Token {
    #[token("(")]
    LParen,
    
    #[token(")")]
    RParen,
    
    #[token("symbol")]
    Symbol,
    
    #[token("property")]
    Property,
    
    #[token("Description")]
    Description,
    
    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    String(String),
    
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_\-\.]*", |lex| lex.slice().to_string())]
    Ident(String),
    
    #[regex(r"-?\d+(\.\d+)?", |lex| lex.slice().parse::<f64>().ok())]
    Number(f64),
}

/// Parse a KiCad symbol library file
pub fn parse_symbol_lib(content: &str) -> Result<Vec<Symbol>> {
    let mut lex = Token::lexer(content);
    let mut symbols = Vec::new();
    
    while let Some(token) = lex.next() {
        match token {
            Ok(Token::LParen) => {
                if let Some(Ok(Token::Symbol)) = lex.next() {
                    if let Some(symbol) = parse_symbol(&mut lex)? {
                        symbols.push(symbol);
                    }
                }
            }
            Ok(_) => {
                // Skip other tokens at top level
            }
            Err(_) => {
                // Skip lexing errors
            }
        }
    }
    
    Ok(symbols)
}

fn parse_symbol(lex: &mut logos::Lexer<Token>) -> Result<Option<Symbol>> {
    // Expect symbol name (either string or identifier)
    let symbol_name = match lex.next() {
        Some(Ok(Token::String(s))) => {
            // Extract base name from full symbol path
            s.split('_').next().unwrap_or(&s).to_string()
        }
        Some(Ok(Token::Ident(s))) => {
            s.split('_').next().unwrap_or(&s).to_string()
        }
        _ => return Err(KicadError::ParseError("Expected symbol name".to_string())),
    };
    
    let mut symbol = Symbol {
        name: symbol_name,
        description: String::new(),
    };
    
    let mut depth = 1;
    
    // Parse symbol contents
    while depth > 0 {
        match lex.next() {
            Some(Ok(Token::LParen)) => {
                depth += 1;
                
                // Check if this is a property element
                if let Some(Ok(Token::Property)) = lex.next() {
                    depth -= 1; // We'll handle the closing paren in parse_property
                    if let Some(description) = parse_property(lex)? {
                        if symbol.description.is_empty() {
                            symbol.description = description;
                        }
                    }
                } else {
                    // Skip other elements by consuming tokens until balanced
                    skip_element(lex, &mut depth)?;
                }
            }
            Some(Ok(Token::RParen)) => {
                depth -= 1;
            }
            Some(Ok(_)) => {
                // Skip other tokens
            }
            Some(Err(_)) => {
                // Skip lexing errors
            }
            None => {
                return Err(KicadError::ParseError("Unexpected end of input".to_string()));
            }
        }
    }
    
    Ok(Some(symbol))
}

fn parse_property(lex: &mut logos::Lexer<Token>) -> Result<Option<String>> {
    // Expect property name
    let property_name = match lex.next() {
        Some(Ok(Token::String(s))) => s,
        Some(Ok(Token::Ident(s))) => s,
        _ => return Ok(None),
    };
    
    // Check if this is a Description property
    if property_name == "Description" {
        // Expect property value
        if let Some(Ok(Token::String(description))) = lex.next() {
            // Skip to closing paren
            let mut depth = 1;
            while depth > 0 {
                match lex.next() {
                    Some(Ok(Token::LParen)) => depth += 1,
                    Some(Ok(Token::RParen)) => depth -= 1,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => {}
                    None => break,
                }
            }
            return Ok(Some(description));
        }
    }
    
    // Skip non-Description properties
    let mut depth = 1;
    while depth > 0 {
        match lex.next() {
            Some(Ok(Token::LParen)) => depth += 1,
            Some(Ok(Token::RParen)) => depth -= 1,
            Some(Ok(_)) => {}
            Some(Err(_)) => {}
            None => break,
        }
    }
    
    Ok(None)
}

fn skip_element(lex: &mut logos::Lexer<Token>, depth: &mut i32) -> Result<()> {
    while *depth > 0 {
        match lex.next() {
            Some(Ok(Token::LParen)) => *depth += 1,
            Some(Ok(Token::RParen)) => *depth -= 1,
            Some(Ok(_)) => {}
            Some(Err(_)) => {}
            None => break,
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_symbol() {
        let content = r#"
        (symbol "Resistor" 
          (property "Description" "Basic resistor component")
        )
        "#;
        
        let symbols = parse_symbol_lib(content).unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Resistor");
        assert_eq!(symbols[0].description, "Basic resistor component");
    }
    
    #[test]
    fn test_symbol_with_variant() {
        let content = r#"
        (symbol "Resistor_SMD_0805" 
          (property "Description" "0805 SMD resistor")
        )
        "#;
        
        let symbols = parse_symbol_lib(content).unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Resistor"); // Base name extracted
        assert_eq!(symbols[0].description, "0805 SMD resistor");
    }
    
    #[test]
    fn test_multiple_symbols() {
        let content = r#"
        (symbol "Resistor" 
          (property "Description" "Basic resistor")
        )
        (symbol "Capacitor"
          (property "Description" "Basic capacitor")
        )
        "#;
        
        let symbols = parse_symbol_lib(content).unwrap();
        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].name, "Resistor");
        assert_eq!(symbols[1].name, "Capacitor");
    }
    
    #[test]
    fn test_symbol_without_description() {
        let content = r#"
        (symbol "Unknown" 
          (property "Value" "Some value")
        )
        "#;
        
        let symbols = parse_symbol_lib(content).unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Unknown");
        assert_eq!(symbols[0].description, "");
    }
}