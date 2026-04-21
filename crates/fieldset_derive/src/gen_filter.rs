use proc_macro2::TokenStream;
use quote::quote;

use crate::parse::{FieldInfo, FieldType, FilterAttr, NumericLit};

/// Generate the body of `fn filter(self) -> Result<Self, FieldsetViolations>`.
pub fn gen_filter(fields: &[FieldInfo]) -> TokenStream {
  let field_filters: Vec<TokenStream> = fields.iter().map(gen_field_filter).collect();

  let field_names: Vec<&syn::Ident> = fields.iter().map(|f| &f.ident).collect();

  quote! {
    fn filter(self) -> ::core::result::Result<Self, walrs_validation::FieldsetViolations> {
      #(#field_filters)*
      ::core::result::Result::Ok(Self {
        #(#field_names),*
      })
    }
  }
}

fn gen_field_filter(field: &FieldInfo) -> TokenStream {
  let field_name = &field.ident;
  let _field_name_str = field_name.to_string();

  // No filters and not nested → passthrough
  if field.filters.is_empty() && !field.is_nested_filter {
    return quote! { let #field_name = self.#field_name; };
  }

  // Nested filter
  if field.is_nested_filter {
    return gen_nested_filter(field);
  }

  // Has filters
  let has_try_filters = field.filters.iter().any(|f| {
    matches!(
      f,
      FilterAttr::TryCustom(_)
        | FilterAttr::ToBool
        | FilterAttr::ToInt
        | FilterAttr::ToFloat
        | FilterAttr::UrlDecode
    )
  });

  match &field.ty {
    FieldType::String => gen_string_filter(field, has_try_filters),
    FieldType::Numeric(ty_ident) => gen_numeric_filter(field, ty_ident),
    FieldType::OptionString => gen_option_string_filter(field, has_try_filters),
    FieldType::OptionNumeric(ty_ident) => gen_option_numeric_filter(field, ty_ident),
    _ => {
      // For other types just passthrough
      quote! { let #field_name = self.#field_name; }
    }
  }
}

fn gen_string_filter(field: &FieldInfo, has_try: bool) -> TokenStream {
  let field_name = &field.ident;
  let field_name_str = field_name.to_string();
  let steps = gen_filter_steps(field, quote! { self.#field_name }, has_try, &field_name_str);

  quote! {
    let #field_name = {
      #steps
    };
  }
}

fn gen_numeric_filter(field: &FieldInfo, ty_ident: &syn::Ident) -> TokenStream {
  let field_name = &field.ident;
  let mut steps = Vec::new();
  let mut current = quote! { self.#field_name };

  for filter in &field.filters {
    let apply = numeric_filter_token(filter, &current, ty_ident);
    current = quote! { filtered };
    steps.push(quote! { let filtered = #apply; });
  }

  if steps.is_empty() {
    quote! { let #field_name = self.#field_name; }
  } else {
    quote! {
      let #field_name = {
        #(#steps)*
        filtered
      };
    }
  }
}

fn gen_option_string_filter(field: &FieldInfo, has_try: bool) -> TokenStream {
  let field_name = &field.ident;
  let field_name_str = field_name.to_string();
  let inner_steps = gen_filter_steps(field, quote! { v }, has_try, &field_name_str);

  if has_try {
    quote! {
      let #field_name = match self.#field_name {
        ::core::option::Option::Some(v) => {
          let result = (|| -> ::core::result::Result<String, walrs_validation::FieldsetViolations> {
            ::core::result::Result::Ok({ #inner_steps })
          })();
          match result {
            ::core::result::Result::Ok(filtered) => ::core::option::Option::Some(filtered),
            ::core::result::Result::Err(e) => return ::core::result::Result::Err(e),
          }
        }
        ::core::option::Option::None => ::core::option::Option::None,
      };
    }
  } else {
    quote! {
      let #field_name = match self.#field_name {
        ::core::option::Option::Some(v) => {
          ::core::option::Option::Some({ #inner_steps })
        }
        ::core::option::Option::None => ::core::option::Option::None,
      };
    }
  }
}

fn gen_option_numeric_filter(field: &FieldInfo, ty_ident: &syn::Ident) -> TokenStream {
  let field_name = &field.ident;
  let mut steps = Vec::new();
  let mut current = quote! { v };

  for filter in &field.filters {
    let apply = numeric_filter_token(filter, &current, ty_ident);
    current = quote! { filtered };
    steps.push(quote! { let filtered = #apply; });
  }

  if steps.is_empty() {
    quote! { let #field_name = self.#field_name; }
  } else {
    quote! {
      let #field_name = match self.#field_name {
        ::core::option::Option::Some(v) => {
          #(#steps)*
          ::core::option::Option::Some(filtered)
        }
        ::core::option::Option::None => ::core::option::Option::None,
      };
    }
  }
}

fn gen_nested_filter(field: &FieldInfo) -> TokenStream {
  let field_name = &field.ident;
  let field_name_str = field_name.to_string();

  match &field.ty {
    FieldType::OptionOther(_) | FieldType::OptionString => {
      quote! {
        let #field_name = match self.#field_name {
          ::core::option::Option::Some(v) => {
            match walrs_fieldfilter::Fieldset::filter(v) {
              ::core::result::Result::Ok(filtered) => ::core::option::Option::Some(filtered),
              ::core::result::Result::Err(e) => {
                let mut fv = walrs_validation::FieldsetViolations::new();
                fv.merge_prefixed(#field_name_str, e);
                return ::core::result::Result::Err(fv);
              }
            }
          }
          ::core::option::Option::None => ::core::option::Option::None,
        };
      }
    }
    _ => {
      quote! {
        let #field_name = walrs_fieldfilter::Fieldset::filter(self.#field_name)
          .map_err(|e| {
            let mut fv = walrs_validation::FieldsetViolations::new();
            fv.merge_prefixed(#field_name_str, e);
            fv
          })?;
      }
    }
  }
}

/// Generate sequential filter application steps for String fields.
fn gen_filter_steps(
  field: &FieldInfo,
  initial: TokenStream,
  _has_try: bool,
  field_name_str: &str,
) -> TokenStream {
  let mut steps = Vec::new();
  let mut first = true;

  for filter in &field.filters {
    let src = if first {
      first = false;
      initial.clone()
    } else {
      quote! { filtered }
    };

    match filter {
      FilterAttr::Trim => {
        steps.push(quote! { let filtered = walrs_filter::FilterOp::<String>::Trim.apply(#src); });
      }
      FilterAttr::Lowercase => {
        steps
          .push(quote! { let filtered = walrs_filter::FilterOp::<String>::Lowercase.apply(#src); });
      }
      FilterAttr::Uppercase => {
        steps
          .push(quote! { let filtered = walrs_filter::FilterOp::<String>::Uppercase.apply(#src); });
      }
      FilterAttr::StripTags => {
        steps
          .push(quote! { let filtered = walrs_filter::FilterOp::<String>::StripTags.apply(#src); });
      }
      FilterAttr::HtmlEntities => {
        steps.push(
          quote! { let filtered = walrs_filter::FilterOp::<String>::HtmlEntities.apply(#src); },
        );
      }
      FilterAttr::Slug { max_length } => {
        let ml = match max_length {
          Some(n) => quote! { ::core::option::Option::Some(#n) },
          None => quote! { ::core::option::Option::None },
        };
        steps.push(
          quote! { let filtered = walrs_filter::FilterOp::<String>::Slug { max_length: #ml }.apply(#src); },
        );
      }
      FilterAttr::Truncate { max_length } => {
        let n = *max_length;
        steps.push(
          quote! { let filtered = walrs_filter::FilterOp::<String>::Truncate { max_length: #n }.apply(#src); },
        );
      }
      FilterAttr::Replace { from, to } => {
        steps.push(
          quote! { let filtered = walrs_filter::FilterOp::<String>::Replace { from: #from.to_string(), to: #to.to_string() }.apply(#src); },
        );
      }
      FilterAttr::Custom(path) => {
        steps.push(
          quote! { let filtered = walrs_filter::FilterOp::<String>::Custom(::std::sync::Arc::new(#path)).apply(#src); },
        );
      }
      FilterAttr::TryCustom(path) => {
        let fname = field_name_str;
        steps.push(quote! {
          let filtered = walrs_filter::TryFilterOp::<String>::TryCustom(::std::sync::Arc::new(#path))
            .try_apply(#src)
            .map_err(|e| {
              let mut fv = walrs_validation::FieldsetViolations::new();
              let violation: walrs_validation::Violation = e.into();
              fv.add(#fname, violation);
              fv
            })?;
        });
      }
      FilterAttr::Digits => {
        steps
          .push(quote! { let filtered = walrs_filter::FilterOp::<String>::Digits.apply(#src); });
      }
      FilterAttr::Alnum { allow_whitespace } => {
        let aw = *allow_whitespace;
        steps.push(
          quote! { let filtered = walrs_filter::FilterOp::<String>::Alnum { allow_whitespace: #aw }.apply(#src); },
        );
      }
      FilterAttr::Alpha { allow_whitespace } => {
        let aw = *allow_whitespace;
        steps.push(
          quote! { let filtered = walrs_filter::FilterOp::<String>::Alpha { allow_whitespace: #aw }.apply(#src); },
        );
      }
      FilterAttr::StripNewlines => {
        steps.push(
          quote! { let filtered = walrs_filter::FilterOp::<String>::StripNewlines.apply(#src); },
        );
      }
      FilterAttr::NormalizeWhitespace => {
        steps.push(
          quote! { let filtered = walrs_filter::FilterOp::<String>::NormalizeWhitespace.apply(#src); },
        );
      }
      FilterAttr::AllowChars { set } => {
        steps.push(
          quote! { let filtered = walrs_filter::FilterOp::<String>::AllowChars { set: #set.to_string() }.apply(#src); },
        );
      }
      FilterAttr::DenyChars { set } => {
        steps.push(
          quote! { let filtered = walrs_filter::FilterOp::<String>::DenyChars { set: #set.to_string() }.apply(#src); },
        );
      }
      FilterAttr::UrlEncode => {
        steps.push(
          quote! { let filtered = walrs_filter::FilterOp::<String>::UrlEncode { encode_unreserved: false }.apply(#src); },
        );
      }
      FilterAttr::ToBool => {
        let fname = field_name_str;
        steps.push(quote! {
          let filtered = walrs_filter::TryFilterOp::<String>::ToBool
            .try_apply(#src)
            .map_err(|e| {
              let mut fv = walrs_validation::FieldsetViolations::new();
              let violation: walrs_validation::Violation = e.into();
              fv.add(#fname, violation);
              fv
            })?;
        });
      }
      FilterAttr::ToInt => {
        let fname = field_name_str;
        steps.push(quote! {
          let filtered = walrs_filter::TryFilterOp::<String>::ToInt
            .try_apply(#src)
            .map_err(|e| {
              let mut fv = walrs_validation::FieldsetViolations::new();
              let violation: walrs_validation::Violation = e.into();
              fv.add(#fname, violation);
              fv
            })?;
        });
      }
      FilterAttr::ToFloat => {
        let fname = field_name_str;
        steps.push(quote! {
          let filtered = walrs_filter::TryFilterOp::<String>::ToFloat
            .try_apply(#src)
            .map_err(|e| {
              let mut fv = walrs_validation::FieldsetViolations::new();
              let violation: walrs_validation::Violation = e.into();
              fv.add(#fname, violation);
              fv
            })?;
        });
      }
      FilterAttr::UrlDecode => {
        let fname = field_name_str;
        steps.push(quote! {
          let filtered = walrs_filter::TryFilterOp::<String>::UrlDecode
            .try_apply(#src)
            .map_err(|e| {
              let mut fv = walrs_validation::FieldsetViolations::new();
              let violation: walrs_validation::Violation = e.into();
              fv.add(#fname, violation);
              fv
            })?;
        });
      }
      FilterAttr::Clamp { .. } => {
        // Clamp doesn't apply to strings; ignore
        if first {
          first = false;
        }
        steps.push(quote! { let filtered = #src; });
      }
    }
  }

  if steps.is_empty() {
    initial
  } else {
    quote! {
      #(#steps)*
      filtered
    }
  }
}

fn numeric_filter_token(
  filter: &FilterAttr,
  src: &TokenStream,
  ty_ident: &syn::Ident,
) -> TokenStream {
  match filter {
    FilterAttr::Clamp { min, max } => {
      let min_lit = numeric_lit_token(min);
      let max_lit = numeric_lit_token(max);
      quote! {
        walrs_filter::FilterOp::<#ty_ident>::Clamp { min: #min_lit, max: #max_lit }.apply(#src)
      }
    }
    FilterAttr::Custom(path) => {
      quote! {
        walrs_filter::FilterOp::<#ty_ident>::Custom(::std::sync::Arc::new(#path)).apply(#src)
      }
    }
    // Other string filters don't apply to numeric types
    _ => quote! { #src },
  }
}

fn numeric_lit_token(n: &NumericLit) -> TokenStream {
  match n {
    NumericLit::Int(v) => {
      let lit = proc_macro2::Literal::i128_unsuffixed(*v);
      quote! { #lit }
    }
    NumericLit::Float(v) => {
      let lit = proc_macro2::Literal::f64_unsuffixed(*v);
      quote! { #lit }
    }
  }
}
