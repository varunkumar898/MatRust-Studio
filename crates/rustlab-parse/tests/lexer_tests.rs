use rustlab_parse::{tokenize, TokenKind};

#[test]
fn test_numeric_literals() {
    let code = "42 3.25 .5 1e-4 2.5E+3 0xFF 0b1010 3i 4.5j";
    let tokens = tokenize(code).unwrap();
    let kinds: Vec<_> = tokens.into_iter().map(|t| t.kind).collect();

    assert_eq!(
        kinds,
        vec![
            TokenKind::Number(42.0),
            TokenKind::Number(3.25),
            TokenKind::Number(0.5),
            TokenKind::Number(1e-4),
            TokenKind::Number(2500.0),
            TokenKind::Number(255.0),
            TokenKind::Number(10.0),
            TokenKind::Imaginary(3.0),
            TokenKind::Imaginary(4.5),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn test_char_and_string_literals() {
    let code = "'hello' 'don''t' \"world\" \"she said \"\"hi\"\"\"";
    let tokens = tokenize(code).unwrap();
    let kinds: Vec<_> = tokens.into_iter().map(|t| t.kind).collect();

    assert_eq!(
        kinds,
        vec![
            TokenKind::CharVector("hello".to_string()),
            TokenKind::CharVector("don't".to_string()),
            TokenKind::StringLiteral("world".to_string()),
            TokenKind::StringLiteral("she said \"hi\"".to_string()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn test_transpose_vs_char_vector() {
    // In A', ' is conjugate transpose
    // In 'text', ' is opening char vector
    // In A.' , .' is array transpose
    let code = "A' + 'text' + (x + y)' + [1 2]' + A.''";
    let tokens = tokenize(code).unwrap();
    let kinds: Vec<_> = tokens.into_iter().map(|t| t.kind).collect();

    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("A".to_string()),
            TokenKind::ConjugateTranspose,
            TokenKind::Plus,
            TokenKind::CharVector("text".to_string()),
            TokenKind::Plus,
            TokenKind::LParen,
            TokenKind::Ident("x".to_string()),
            TokenKind::Plus,
            TokenKind::Ident("y".to_string()),
            TokenKind::RParen,
            TokenKind::ConjugateTranspose,
            TokenKind::Plus,
            TokenKind::LBracket,
            TokenKind::Number(1.0),
            TokenKind::Number(2.0),
            TokenKind::RBracket,
            TokenKind::ConjugateTranspose,
            TokenKind::Plus,
            TokenKind::Ident("A".to_string()),
            TokenKind::Transpose,
            TokenKind::ConjugateTranspose,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn test_comments_and_line_continuations() {
    let code = "
    x = 10; % this is a comment
    y = ... line continuation
        20;
    %{
       Block comment here
    %}
    z = 30;
    ";
    let tokens = tokenize(code).unwrap();
    let idents: Vec<_> = tokens
        .into_iter()
        .filter_map(|t| match t.kind {
            TokenKind::Ident(id) => Some(id),
            _ => None,
        })
        .collect();

    assert_eq!(idents, vec!["x", "y", "z"]);
}
