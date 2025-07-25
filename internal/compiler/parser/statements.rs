// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0

use crate::parser::r#type::parse_type;

use super::element::parse_code_block;
use super::expressions::parse_expression;
use super::prelude::*;

#[cfg_attr(test, parser_test)]
/// ```test
/// expression
/// expression += expression
/// expression.expression *= 45.2
/// expression = "hello"
/// if (true) { foo = bar; } else { bar = foo;  }
/// return;
/// if (true) { return 42; }
/// let foo = 1;
/// let bar = foo;
/// let str: string = "hello world";
/// ```
pub fn parse_statement(p: &mut impl Parser) -> bool {
    if p.nth(0).kind() == SyntaxKind::RBrace {
        return false;
    }
    if p.test(SyntaxKind::Semicolon) {
        return true;
    }
    let checkpoint = p.checkpoint();

    if p.peek().as_str() == "if"
        && !matches!(
            p.nth(1).kind(),
            SyntaxKind::Dot
                | SyntaxKind::Comma
                | SyntaxKind::Semicolon
                | SyntaxKind::RBrace
                | SyntaxKind::RBracket
                | SyntaxKind::RParent
        )
    {
        let mut p = p.start_node(SyntaxKind::Expression);
        parse_if_statement(&mut *p);
        return true;
    }

    if p.peek().as_str() == "return" {
        let mut p = p.start_node_at(checkpoint, SyntaxKind::ReturnStatement);
        p.expect(SyntaxKind::Identifier); // "return"
        if !p.test(SyntaxKind::Semicolon) {
            parse_expression(&mut *p);
            p.expect(SyntaxKind::Semicolon);
        }
        return true;
    }

    if p.peek().as_str() == "let" && p.nth(1).kind() == SyntaxKind::Identifier {
        parse_let_statement(p);
        return true;
    }

    parse_expression(p);
    if matches!(
        p.nth(0).kind(),
        SyntaxKind::MinusEqual
            | SyntaxKind::PlusEqual
            | SyntaxKind::StarEqual
            | SyntaxKind::DivEqual
            | SyntaxKind::Equal
    ) {
        let mut p = p.start_node_at(checkpoint.clone(), SyntaxKind::Expression);
        let mut p = p.start_node_at(checkpoint, SyntaxKind::SelfAssignment);
        p.consume();
        parse_expression(&mut *p);
    }
    p.test(SyntaxKind::Semicolon)
}

#[cfg_attr(test, parser_test)]
/// ```test,LetStatement
/// let foo = 1;
/// let bar = foo;
/// let str: string = "hello world";
/// ```
fn parse_let_statement(p: &mut impl Parser) {
    let mut p = p.start_node(SyntaxKind::LetStatement);
    debug_assert_eq!(p.peek().as_str(), "let");
    p.expect(SyntaxKind::Identifier); // "let"
    {
        let mut p = p.start_node(SyntaxKind::DeclaredIdentifier);
        p.expect(SyntaxKind::Identifier);
    }

    if p.test(SyntaxKind::Colon) {
        parse_type(&mut *p);
    }

    p.expect(SyntaxKind::Equal);
    parse_expression(&mut *p);
    p.expect(SyntaxKind::Semicolon);
}

#[cfg_attr(test, parser_test)]
/// ```test,ConditionalExpression
/// if (true) { foo = bar; } else { bar = foo;  }
/// if (true) { foo += bar; }
/// if (true) { } else { ; }
/// if (true) { } else if (false) { } else if (xxx) { }
/// ```
fn parse_if_statement(p: &mut impl Parser) {
    let mut p = p.start_node(SyntaxKind::ConditionalExpression);
    debug_assert_eq!(p.peek().as_str(), "if");
    p.expect(SyntaxKind::Identifier);
    parse_expression(&mut *p);
    {
        let mut p = p.start_node(SyntaxKind::Expression);
        parse_code_block(&mut *p);
    }
    if p.peek().as_str() == "else" {
        p.expect(SyntaxKind::Identifier);
        let mut p = p.start_node(SyntaxKind::Expression);
        if p.peek().as_str() == "if" {
            parse_if_statement(&mut *p)
        } else {
            parse_code_block(&mut *p);
        }
    } else {
        // We need an expression so fake an empty block.
        // FIXME: this shouldn't be needed
        let mut p = p.start_node(SyntaxKind::Expression);
        let _ = p.start_node(SyntaxKind::CodeBlock);
    }
}
