use crate::{
    Parse, ParseError, Phase, PhaseParse, ProcTerm, ProcTermVariable,
    token::{Token, TokenKeyword, TokenNumber, TokenVariable},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StatementCallPtx<P: Phase> {
    pub keyword_call_ptx: TokenKeyword,
    pub function_name: TokenVariable,
    pub args: Vec<ProcTerm<P>>,
    pub grid_dim_x: TokenNumber,
    pub grid_dim_y: TokenNumber,
    pub grid_dim_z: TokenNumber,
    pub block_dim_x: TokenNumber,
    pub block_dim_y: TokenNumber,
    pub block_dim_z: TokenNumber,
    pub ext: P::StatementCallPtxExt,
}

impl Parse for StatementCallPtx<PhaseParse> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        let mut k = *i;

        let Some(keyword_call_ptx) = TokenKeyword::parse_keyword(tokens, &mut k, "call_ptx")?
        else {
            return Ok(None);
        };

        let Some(function_name) = TokenVariable::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        // Parse arguments (only one array argument expected)
        let mut args = Vec::new();
        if let Some(variable) = TokenVariable::parse(tokens, &mut k)? {
            let proc_term_variable = ProcTermVariable { variable, ext: () };
            args.push(ProcTerm::Variable(proc_term_variable));
        }

        // Parse the 6 dimension parameters
        let Some(grid_dim_x) = TokenNumber::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        let Some(grid_dim_y) = TokenNumber::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        let Some(grid_dim_z) = TokenNumber::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        let Some(block_dim_x) = TokenNumber::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        let Some(block_dim_y) = TokenNumber::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        let Some(block_dim_z) = TokenNumber::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let statement = StatementCallPtx {
            keyword_call_ptx,
            function_name,
            args,
            grid_dim_x,
            grid_dim_y,
            grid_dim_z,
            block_dim_x,
            block_dim_y,
            block_dim_z,
            ext: (),
        };
        *i = k;
        Ok(Some(statement))
    }
}
