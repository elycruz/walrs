use quote::ToTokens;
use syn::{
  Attribute, Expr, ExprLit, Field, Ident, Lit, LitFloat, LitInt, LitStr, MetaNameValue, Path,
  Token, Type, TypePath, parenthesized, parse::Parse, parse::ParseStream, punctuated::Punctuated,
  token,
};

// ---------------------------------------------------------------------------
// Struct-level attributes
// ---------------------------------------------------------------------------

/// Parsed `#[fieldset(...)]` on the struct.
#[derive(Debug, Default)]
pub struct FieldsetStructAttrs {
  pub break_on_failure: bool,
  pub into_form_data: bool,
  pub try_from_form_data: bool,
}

/// Parsed `#[cross_validate(fn_name)]` on the struct.
#[derive(Debug, Default)]
pub struct CrossValidateAttrs {
  pub fns: Vec<Path>,
}

// ---------------------------------------------------------------------------
// Field-level parsed data
// ---------------------------------------------------------------------------

/// Everything we need to know about one field.
#[derive(Debug)]
pub struct FieldInfo {
  pub ident: Ident,
  pub ty: FieldType,
  pub validations: Vec<ValidateAttr>,
  pub filters: Vec<FilterAttr>,
  pub is_nested_validate: bool,
  pub is_nested_filter: bool,
  pub break_on_failure_override: Option<bool>,
}

/// Simplified type classification.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FieldType {
  String,
  Bool,
  Char,
  Numeric(Ident), // i32, u64, f64, etc.
  OptionString,
  OptionBool,
  OptionChar,
  OptionNumeric(Ident),
  Other(Type), // for nested types
  OptionOther(Type),
}

// ---------------------------------------------------------------------------
// Validate attributes
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ValidateAttr {
  Required,
  MinLength(usize),
  MaxLength(usize),
  ExactLength(usize),
  Email,
  Url,
  Uri,
  Ip,
  Hostname,
  Pattern(String),
  Min(NumericLit),
  Max(NumericLit),
  Range { min: NumericLit, max: NumericLit },
  Step(NumericLit),
  OneOf(Vec<OneOfItem>),
  Custom(Path),
  Message(String),
  MessageFn(Path),
  Locale(String),
}

/// A numeric literal that can be integer or float.
#[derive(Debug, Clone)]
pub enum NumericLit {
  Int(i128),
  Float(f64),
}

/// Items in a `one_of = [...]` list.
#[derive(Debug, Clone)]
pub enum OneOfItem {
  Str(String),
  Int(i128),
  Float(f64),
}

// ---------------------------------------------------------------------------
// Filter attributes
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum FilterAttr {
  Trim,
  Lowercase,
  Uppercase,
  StripTags,
  HtmlEntities,
  Slug { max_length: Option<usize> },
  Truncate { max_length: usize },
  Replace { from: String, to: String },
  Clamp { min: NumericLit, max: NumericLit },
  Digits,
  Alnum { allow_whitespace: bool },
  Alpha { allow_whitespace: bool },
  StripNewlines,
  NormalizeWhitespace,
  AllowChars { set: String },
  DenyChars { set: String },
  UrlEncode,
  ToBool,
  ToInt,
  ToFloat,
  UrlDecode,
  Custom(Path),
  TryCustom(Path),
}

// ---------------------------------------------------------------------------
// Parsing implementations
// ---------------------------------------------------------------------------

const NUMERIC_TYPES: &[&str] = &[
  "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
  "f64",
];

/// Classify a syn::Type into our FieldType.
pub fn classify_type(ty: &Type) -> FieldType {
  if let Type::Path(TypePath { path, .. }) = ty
    && let Some(seg) = path.segments.last()
  {
    let name = seg.ident.to_string();
    // Check for Option<T>
    if name == "Option" {
      if let syn::PathArguments::AngleBracketed(args) = &seg.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
      {
        return classify_option_inner(inner, ty);
      }
      return FieldType::OptionOther(ty.clone());
    }
    if name == "String" {
      return FieldType::String;
    }
    if name == "bool" {
      return FieldType::Bool;
    }
    if name == "char" {
      return FieldType::Char;
    }
    if NUMERIC_TYPES.contains(&name.as_str()) {
      return FieldType::Numeric(seg.ident.clone());
    }
  }
  FieldType::Other(ty.clone())
}

fn classify_option_inner(inner: &Type, _outer: &Type) -> FieldType {
  if let Type::Path(TypePath { path, .. }) = inner
    && let Some(seg) = path.segments.last()
  {
    let name = seg.ident.to_string();
    if name == "String" {
      return FieldType::OptionString;
    }
    if name == "bool" {
      return FieldType::OptionBool;
    }
    if name == "char" {
      return FieldType::OptionChar;
    }
    if NUMERIC_TYPES.contains(&name.as_str()) {
      return FieldType::OptionNumeric(seg.ident.clone());
    }
  }
  FieldType::OptionOther(inner.clone())
}

// ---------------------------------------------------------------------------
// Parse struct-level `#[fieldset(...)]`
// ---------------------------------------------------------------------------

pub fn parse_fieldset_struct_attrs(attrs: &[Attribute]) -> FieldsetStructAttrs {
  let mut result = FieldsetStructAttrs::default();
  for attr in attrs {
    if attr.path().is_ident("fieldset") {
      let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("break_on_failure") {
          result.break_on_failure = true;
        } else if meta.path.is_ident("into_form_data") {
          result.into_form_data = true;
        } else if meta.path.is_ident("try_from_form_data") {
          result.try_from_form_data = true;
        }
        Ok(())
      });
    }
  }
  result
}

// ---------------------------------------------------------------------------
// Parse struct-level `#[cross_validate(fn_name)]`
// ---------------------------------------------------------------------------

pub fn parse_cross_validate_attrs(attrs: &[Attribute]) -> CrossValidateAttrs {
  let mut result = CrossValidateAttrs::default();
  for attr in attrs {
    if attr.path().is_ident("cross_validate")
      && let Ok(path) = attr.parse_args::<Path>()
    {
      result.fns.push(path);
    }
  }
  result
}

// ---------------------------------------------------------------------------
// Parse field-level attributes
// ---------------------------------------------------------------------------

pub fn parse_field_info(field: &Field) -> syn::Result<FieldInfo> {
  let ident = field
    .ident
    .clone()
    .ok_or_else(|| syn::Error::new_spanned(field, "Fieldset derive only supports named fields"))?;
  let ty = classify_type(&field.ty);
  let mut validations = Vec::new();
  let mut filters = Vec::new();
  let mut is_nested_validate = false;
  let mut is_nested_filter = false;
  let mut break_on_failure_override = None;

  for attr in &field.attrs {
    if attr.path().is_ident("validate") {
      parse_validate_attr(attr, &mut validations, &mut is_nested_validate)?;
    } else if attr.path().is_ident("filter") {
      parse_filter_attr(attr, &mut filters, &mut is_nested_filter)?;
    } else if attr.path().is_ident("fieldset") {
      let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("break_on_failure") {
          if meta.input.peek(Token![=]) {
            let _: Token![=] = meta.input.parse()?;
            let lit: syn::LitBool = meta.input.parse()?;
            break_on_failure_override = Some(lit.value());
          } else {
            break_on_failure_override = Some(true);
          }
        }
        Ok(())
      });
    }
  }

  Ok(FieldInfo {
    ident,
    ty,
    validations,
    filters,
    is_nested_validate,
    is_nested_filter,
    break_on_failure_override,
  })
}

fn parse_validate_attr(
  attr: &Attribute,
  validations: &mut Vec<ValidateAttr>,
  is_nested: &mut bool,
) -> syn::Result<()> {
  attr.parse_nested_meta(|meta| {
    let path = &meta.path;

    if path.is_ident("required") {
      validations.push(ValidateAttr::Required);
    } else if path.is_ident("min_length") {
      let _: Token![=] = meta.input.parse()?;
      let lit: LitInt = meta.input.parse()?;
      validations.push(ValidateAttr::MinLength(lit.base10_parse()?));
    } else if path.is_ident("max_length") {
      let _: Token![=] = meta.input.parse()?;
      let lit: LitInt = meta.input.parse()?;
      validations.push(ValidateAttr::MaxLength(lit.base10_parse()?));
    } else if path.is_ident("exact_length") {
      let _: Token![=] = meta.input.parse()?;
      let lit: LitInt = meta.input.parse()?;
      validations.push(ValidateAttr::ExactLength(lit.base10_parse()?));
    } else if path.is_ident("email") {
      validations.push(ValidateAttr::Email);
    } else if path.is_ident("url") {
      validations.push(ValidateAttr::Url);
    } else if path.is_ident("uri") {
      validations.push(ValidateAttr::Uri);
    } else if path.is_ident("ip") {
      validations.push(ValidateAttr::Ip);
    } else if path.is_ident("hostname") {
      validations.push(ValidateAttr::Hostname);
    } else if path.is_ident("pattern") {
      let _: Token![=] = meta.input.parse()?;
      let lit: LitStr = meta.input.parse()?;
      let pat = lit.value();
      if regex::Regex::new(&pat).is_err() {
        return Err(syn::Error::new_spanned(
          &lit,
          format!("invalid regex pattern: \"{}\"", pat),
        ));
      }
      validations.push(ValidateAttr::Pattern(pat));
    } else if path.is_ident("min") {
      let _: Token![=] = meta.input.parse()?;
      validations.push(ValidateAttr::Min(parse_numeric_lit(&meta.input)?));
    } else if path.is_ident("max") {
      let _: Token![=] = meta.input.parse()?;
      validations.push(ValidateAttr::Max(parse_numeric_lit(&meta.input)?));
    } else if path.is_ident("range") {
      let content;
      parenthesized!(content in meta.input);
      let mut min = None;
      let mut max = None;
      let items: Punctuated<MetaNameValue, Token![,]> =
        content.parse_terminated(MetaNameValue::parse, Token![,])?;
      for item in items {
        if item.path.is_ident("min") {
          min = Some(expr_to_numeric_lit(&item.value)?);
        } else if item.path.is_ident("max") {
          max = Some(expr_to_numeric_lit(&item.value)?);
        }
      }
      validations.push(ValidateAttr::Range {
        min: min.ok_or_else(|| syn::Error::new_spanned(path, "range requires `min`"))?,
        max: max.ok_or_else(|| syn::Error::new_spanned(path, "range requires `max`"))?,
      });
    } else if path.is_ident("step") {
      let _: Token![=] = meta.input.parse()?;
      validations.push(ValidateAttr::Step(parse_numeric_lit(&meta.input)?));
    } else if path.is_ident("one_of") {
      let _: Token![=] = meta.input.parse()?;
      let content;
      syn::bracketed!(content in meta.input);
      let items: Punctuated<Expr, Token![,]> = content.parse_terminated(Expr::parse, Token![,])?;
      let mut one_of_items = Vec::new();
      for item in items {
        match &item {
          Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
          }) => one_of_items.push(OneOfItem::Str(s.value())),
          Expr::Lit(ExprLit {
            lit: Lit::Int(i), ..
          }) => one_of_items.push(OneOfItem::Int(i.base10_parse()?)),
          Expr::Lit(ExprLit {
            lit: Lit::Float(f), ..
          }) => one_of_items.push(OneOfItem::Float(f.base10_parse()?)),
          // Handle negative numeric literals like -1
          Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => {
            if let Expr::Lit(ExprLit { lit, .. }) = &*unary.expr {
              match lit {
                Lit::Int(i) => {
                  let val: i128 = i.base10_parse()?;
                  one_of_items.push(OneOfItem::Int(-val));
                }
                Lit::Float(f) => {
                  let val: f64 = f.base10_parse()?;
                  one_of_items.push(OneOfItem::Float(-val));
                }
                _ => {
                  return Err(syn::Error::new_spanned(lit, "Expected numeric literal"));
                }
              }
            }
          }
          _ => return Err(syn::Error::new_spanned(item, "Expected literal in one_of")),
        }
      }
      validations.push(ValidateAttr::OneOf(one_of_items));
    } else if path.is_ident("custom") {
      let _: Token![=] = meta.input.parse()?;
      let lit: LitStr = meta.input.parse()?;
      let path: Path = lit.parse()?;
      validations.push(ValidateAttr::Custom(path));
    } else if path.is_ident("nested") {
      *is_nested = true;
    } else if path.is_ident("message") {
      let _: Token![=] = meta.input.parse()?;
      let lit: LitStr = meta.input.parse()?;
      validations.push(ValidateAttr::Message(lit.value()));
    } else if path.is_ident("message_fn") {
      let _: Token![=] = meta.input.parse()?;
      let lit: LitStr = meta.input.parse()?;
      let path: Path = lit.parse()?;
      validations.push(ValidateAttr::MessageFn(path));
    } else if path.is_ident("locale") {
      let _: Token![=] = meta.input.parse()?;
      let lit: LitStr = meta.input.parse()?;
      validations.push(ValidateAttr::Locale(lit.value()));
    } else {
      return Err(syn::Error::new_spanned(
        path,
        format!("Unknown validate attribute: {}", format_meta_path(path)),
      ));
    }
    Ok(())
  })
}

fn parse_filter_attr(
  attr: &Attribute,
  filters: &mut Vec<FilterAttr>,
  is_nested: &mut bool,
) -> syn::Result<()> {
  attr.parse_nested_meta(|meta| {
    let path = &meta.path;

    if path.is_ident("trim") {
      filters.push(FilterAttr::Trim);
    } else if path.is_ident("lowercase") {
      filters.push(FilterAttr::Lowercase);
    } else if path.is_ident("uppercase") {
      filters.push(FilterAttr::Uppercase);
    } else if path.is_ident("strip_tags") {
      filters.push(FilterAttr::StripTags);
    } else if path.is_ident("html_entities") {
      filters.push(FilterAttr::HtmlEntities);
    } else if path.is_ident("slug") {
      if meta.input.peek(token::Paren) {
        let content;
        parenthesized!(content in meta.input);
        let items: Punctuated<MetaNameValue, Token![,]> =
          content.parse_terminated(MetaNameValue::parse, Token![,])?;
        let mut max_length = None;
        for item in items {
          if item.path.is_ident("max_length") {
            max_length = Some(expr_to_usize(&item.value)?);
          }
        }
        filters.push(FilterAttr::Slug { max_length });
      } else {
        filters.push(FilterAttr::Slug { max_length: None });
      }
    } else if path.is_ident("truncate") {
      let content;
      parenthesized!(content in meta.input);
      let items: Punctuated<MetaNameValue, Token![,]> =
        content.parse_terminated(MetaNameValue::parse, Token![,])?;
      let mut max_length = None;
      for item in items {
        if item.path.is_ident("max_length") {
          max_length = Some(expr_to_usize(&item.value)?);
        }
      }
      filters.push(FilterAttr::Truncate {
        max_length: max_length
          .ok_or_else(|| syn::Error::new_spanned(path, "truncate requires `max_length`"))?,
      });
    } else if path.is_ident("replace") {
      let content;
      parenthesized!(content in meta.input);
      let items: Punctuated<MetaNameValue, Token![,]> =
        content.parse_terminated(MetaNameValue::parse, Token![,])?;
      let mut from = None;
      let mut to = None;
      for item in items {
        if item.path.is_ident("from") {
          from = Some(expr_to_string(&item.value)?);
        } else if item.path.is_ident("to") {
          to = Some(expr_to_string(&item.value)?);
        }
      }
      filters.push(FilterAttr::Replace {
        from: from.ok_or_else(|| syn::Error::new_spanned(path, "replace requires `from`"))?,
        to: to.ok_or_else(|| syn::Error::new_spanned(path, "replace requires `to`"))?,
      });
    } else if path.is_ident("clamp") {
      let content;
      parenthesized!(content in meta.input);
      let items: Punctuated<MetaNameValue, Token![,]> =
        content.parse_terminated(MetaNameValue::parse, Token![,])?;
      let mut min = None;
      let mut max = None;
      for item in items {
        if item.path.is_ident("min") {
          min = Some(expr_to_numeric_lit(&item.value)?);
        } else if item.path.is_ident("max") {
          max = Some(expr_to_numeric_lit(&item.value)?);
        }
      }
      filters.push(FilterAttr::Clamp {
        min: min.ok_or_else(|| syn::Error::new_spanned(path, "clamp requires `min`"))?,
        max: max.ok_or_else(|| syn::Error::new_spanned(path, "clamp requires `max`"))?,
      });
    } else if path.is_ident("digits") {
      filters.push(FilterAttr::Digits);
    } else if path.is_ident("alnum") {
      let allow_whitespace = parse_whitespace_flag(&meta)?;
      filters.push(FilterAttr::Alnum { allow_whitespace });
    } else if path.is_ident("alpha") {
      let allow_whitespace = parse_whitespace_flag(&meta)?;
      filters.push(FilterAttr::Alpha { allow_whitespace });
    } else if path.is_ident("strip_newlines") {
      filters.push(FilterAttr::StripNewlines);
    } else if path.is_ident("normalize_whitespace") {
      filters.push(FilterAttr::NormalizeWhitespace);
    } else if path.is_ident("allow_chars") {
      let _: Token![=] = meta.input.parse()?;
      let lit: LitStr = meta.input.parse()?;
      filters.push(FilterAttr::AllowChars { set: lit.value() });
    } else if path.is_ident("deny_chars") {
      let _: Token![=] = meta.input.parse()?;
      let lit: LitStr = meta.input.parse()?;
      filters.push(FilterAttr::DenyChars { set: lit.value() });
    } else if path.is_ident("url_encode") {
      filters.push(FilterAttr::UrlEncode);
    } else if path.is_ident("to_bool") {
      filters.push(FilterAttr::ToBool);
    } else if path.is_ident("to_int") {
      filters.push(FilterAttr::ToInt);
    } else if path.is_ident("to_float") {
      filters.push(FilterAttr::ToFloat);
    } else if path.is_ident("url_decode") {
      filters.push(FilterAttr::UrlDecode);
    } else if path.is_ident("custom") {
      let _: Token![=] = meta.input.parse()?;
      let lit: LitStr = meta.input.parse()?;
      let path: Path = lit.parse()?;
      filters.push(FilterAttr::Custom(path));
    } else if path.is_ident("try_custom") {
      let _: Token![=] = meta.input.parse()?;
      let lit: LitStr = meta.input.parse()?;
      let path: Path = lit.parse()?;
      filters.push(FilterAttr::TryCustom(path));
    } else if path.is_ident("nested") {
      *is_nested = true;
    } else {
      return Err(syn::Error::new_spanned(
        path,
        format!("Unknown filter attribute: {}", format_meta_path(path)),
      ));
    }
    Ok(())
  })
}

/// Parse an optional `(whitespace)` flag for `alnum` / `alpha` attributes.
///
/// Accepts:
/// - `alnum` → `false`
/// - `alnum(whitespace)` → `true`
fn parse_whitespace_flag(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<bool> {
  if !meta.input.peek(token::Paren) {
    return Ok(false);
  }
  let content;
  parenthesized!(content in meta.input);
  if content.is_empty() {
    return Err(content.error("expected `whitespace` inside parentheses"));
  }
  let idents: Punctuated<Ident, Token![,]> = content.parse_terminated(Ident::parse, Token![,])?;
  let mut allow_whitespace = false;
  for id in idents {
    if id == "whitespace" {
      allow_whitespace = true;
    } else {
      return Err(syn::Error::new_spanned(
        &id,
        format!("Unknown flag: {id}; expected `whitespace`"),
      ));
    }
  }
  Ok(allow_whitespace)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_numeric_lit(input: &ParseStream) -> syn::Result<NumericLit> {
  // Check for negative sign
  let negative = if input.peek(Token![-]) {
    let _: Token![-] = input.parse()?;
    true
  } else {
    false
  };

  if input.peek(LitFloat) {
    let lit: LitFloat = input.parse()?;
    let val: f64 = lit.base10_parse()?;
    Ok(NumericLit::Float(if negative { -val } else { val }))
  } else if input.peek(LitInt) {
    let lit: LitInt = input.parse()?;
    let val: i128 = lit.base10_parse()?;
    Ok(NumericLit::Int(if negative { -val } else { val }))
  } else {
    Err(input.error("Expected numeric literal"))
  }
}

fn expr_to_numeric_lit(expr: &Expr) -> syn::Result<NumericLit> {
  match expr {
    Expr::Lit(ExprLit {
      lit: Lit::Int(i), ..
    }) => Ok(NumericLit::Int(i.base10_parse()?)),
    Expr::Lit(ExprLit {
      lit: Lit::Float(f), ..
    }) => Ok(NumericLit::Float(f.base10_parse()?)),
    Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => {
      if let Expr::Lit(ExprLit { lit, .. }) = &*unary.expr {
        match lit {
          Lit::Int(i) => {
            let val: i128 = i.base10_parse()?;
            Ok(NumericLit::Int(-val))
          }
          Lit::Float(f) => {
            let val: f64 = f.base10_parse()?;
            Ok(NumericLit::Float(-val))
          }
          _ => Err(syn::Error::new_spanned(expr, "Expected numeric literal")),
        }
      } else {
        Err(syn::Error::new_spanned(expr, "Expected numeric literal"))
      }
    }
    _ => Err(syn::Error::new_spanned(expr, "Expected numeric literal")),
  }
}

fn expr_to_usize(expr: &Expr) -> syn::Result<usize> {
  if let Expr::Lit(ExprLit {
    lit: Lit::Int(i), ..
  }) = expr
  {
    i.base10_parse()
  } else {
    Err(syn::Error::new_spanned(expr, "Expected integer literal"))
  }
}

fn expr_to_string(expr: &Expr) -> syn::Result<String> {
  if let Expr::Lit(ExprLit {
    lit: Lit::Str(s), ..
  }) = expr
  {
    Ok(s.value())
  } else {
    Err(syn::Error::new_spanned(expr, "Expected string literal"))
  }
}

fn format_meta_path(path: &Path) -> String {
  path
    .get_ident()
    .map(ToString::to_string)
    .unwrap_or_else(|| path.to_token_stream().to_string())
}

#[cfg(test)]
mod tests {
  use super::*;
  use quote::quote;
  use syn::{Fields, ItemStruct, parse_str};

  /// Parse a struct declaration and return its first named field.
  fn parse_named_field(src: &str) -> Field {
    let item: ItemStruct = parse_str(src).expect("valid struct");
    match item.fields {
      Fields::Named(named) => named.named.into_iter().next().expect("one field"),
      _ => panic!("expected named fields"),
    }
  }

  /// Parse a struct with quote! macro and return its first named field.
  fn parse_named_field_from_tokens(tokens: proc_macro2::TokenStream) -> syn::Field {
    let item: ItemStruct = syn::parse2(tokens).expect("struct should parse");
    match item.fields {
      Fields::Named(fields) => fields
        .named
        .into_iter()
        .next()
        .expect("struct should have one field"),
      _ => panic!("expected named fields"),
    }
  }

  #[test]
  fn parse_field_info_rejects_unknown_validate_attrs() {
    let field = parse_named_field("struct S { #[validate(nonsense)] x: String }");
    let err = parse_field_info(&field).expect_err("unknown validate attr should error");
    assert!(
      err.to_string().contains("Unknown validate attribute"),
      "expected 'Unknown validate attribute' in error, got: {}",
      err
    );
    assert!(
      err.to_string().contains("nonsense") && !err.to_string().contains("Some("),
      "expected unknown key name in error without Option debug formatting, got: {}",
      err
    );
  }

  #[test]
  fn parse_field_info_rejects_invalid_regex_pattern() {
    let field = parse_named_field(r#"struct S { #[validate(pattern = "[")] x: String }"#);
    let err = parse_field_info(&field).expect_err("invalid regex should error");
    assert!(
      err.to_string().contains("invalid regex pattern"),
      "expected 'invalid regex pattern' in error, got: {}",
      err
    );
  }

  #[test]
  fn parse_field_info_rejects_unknown_filter_attrs() {
    let field = parse_named_field_from_tokens(quote! {
      struct Example {
        #[filter(unknown)]
        value: String
      }
    });

    let err = parse_field_info(&field).expect_err("unknown filter should error");
    assert!(err.to_string().contains("Unknown filter attribute"));
  }

  #[test]
  fn parse_field_info_rejects_empty_whitespace_flag() {
    let field = parse_named_field_from_tokens(quote! {
      struct Example {
        #[filter(alnum())]
        value: String
      }
    });

    let err = parse_field_info(&field).expect_err("empty alnum parens should error");
    assert!(
      err
        .to_string()
        .contains("expected `whitespace` inside parentheses")
    );
  }
}
