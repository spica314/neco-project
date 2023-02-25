use crate::{
    parse::Parse,
    syn_expr::SynExpr,
    syn_type::SynType,
    syn_typed_arg::SynTypedArg,
    token::{Token, TokenArrow, TokenIdent, TokenKeyword, TokenLBrace, TokenRBrace},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynFnDef {
    pub keyword_fn: TokenKeyword,
    pub name: TokenIdent,
    pub args: Vec<SynTypedArg>,
    pub arrow: TokenArrow,
    pub res_ty: SynType,
    pub fn_block: SynFnBlock,
}

impl Parse for SynFnDef {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(keyword_fn) = TokenKeyword::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        if keyword_fn.keyword != "#fn" {
            return Ok(None);
        };

        let Some(name) = TokenIdent::parse(tokens, &mut k)? else {
            return Err(());
        };

        let mut args = vec![];
        while let Some(arg) = SynTypedArg::parse(tokens, &mut k)? {
            args.push(arg);
        }

        let Some(arrow) = TokenArrow::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(res_ty) = SynType::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(fn_block) = SynFnBlock::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynFnDef {
            keyword_fn,
            name,
            args,
            arrow,
            res_ty,
            fn_block,
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynFnBlock {
    pub lbrace: TokenLBrace,
    pub statements: Vec<SynStatement>,
    pub rbrace: TokenRBrace,
}

impl Parse for SynFnBlock {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(lbrace) = TokenLBrace::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let mut statements = vec![];
        while let Some(statement) = SynStatement::parse(tokens, &mut k)? {
            statements.push(statement);
        }

        let Some(rbrace) = TokenRBrace::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynFnBlock {
            lbrace,
            statements,
            rbrace,
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SynStatement {
    Expr(SynExpr),
}

impl Parse for SynStatement {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        if let Some(expr) = SynExpr::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynStatement::Expr(expr)));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use crate::{to_felis_string::ToFelisString, Parser};

    use super::*;

    #[test]
    fn felis_syn_fn_def_parse_test_1() {
        let s = "#fn f -> T { }";
        let mut parser = Parser::new();
        let res = parser.parse::<SynFnDef>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.name.ident.as_str(), "f");
        assert_eq!(res.args.len(), 0);
        assert_eq!(res.fn_block.statements.len(), 0);
    }

    #[test]
    fn felis_syn_fn_def_parse_test_2() {
        let s = "#fn f -> T { x }";
        let mut parser = Parser::new();
        let res = parser.parse::<SynFnDef>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.name.ident.as_str(), "f");
        assert_eq!(res.args.len(), 0);
        assert_eq!(res.fn_block.statements.len(), 1);
        // todo: check expr
    }

    #[test]
    fn felis_syn_fn_def_parse_test_3() {
        let s = "#fn f (x : T) -> T { x }";
        let mut parser = Parser::new();
        let res = parser.parse::<SynFnDef>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.name.ident.as_str(), "f");
        assert_eq!(res.args.len(), 1);
        assert_eq!(res.args[0].to_felis_string(), "(x : T)");
        assert_eq!(res.fn_block.statements.len(), 1);
        // todo: check expr
    }

    #[test]
    fn felis_syn_fn_def_parse_test_4() {
        let s = std::fs::read_to_string("../../library/wip/fn_def.fe").unwrap();
        let mut parser = Parser::new();
        let res = parser.parse::<SynFnDef>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.name.ident.as_str(), "proof");
        assert_eq!(res.args.len(), 3);
        assert_eq!(res.args[0].to_felis_string(), "(A : Prop)");
        assert_eq!(res.args[1].to_felis_string(), "(B : Prop)");
        assert_eq!(res.args[2].to_felis_string(), "(x : And A B)");
        assert_eq!(res.res_ty.to_felis_string(), "Or A B");
        assert_eq!(res.fn_block.statements.len(), 1);
        // todo: check expr
    }
}
