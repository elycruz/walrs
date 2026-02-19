#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rule {
  Allow = 0,
  Deny = 1,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RuleContextScope {
  PerSymbol,
  ForAllSymbols,
}
