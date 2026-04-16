mod gen_filter;
mod gen_validate;
mod parse;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

use gen_filter::gen_filter;
use gen_validate::gen_validate;
use parse::{parse_cross_validate_attrs, parse_field_info, parse_fieldset_struct_attrs};

/// Derive macro for the `Fieldset` trait.
///
/// # Attributes
///
/// ## Struct-level
///
/// - `#[fieldset(break_on_failure)]` — stop validation after the first field with violations
/// - `#[cross_validate(fn_name)]` — call `fn_name(&self) -> RuleResult` after per-field validation
///
/// ## Field-level validation (`#[validate(...)]`)
///
/// - `required` — field must not be empty/missing
/// - `min_length = N` — minimum string/collection length
/// - `max_length = N` — maximum string/collection length
/// - `exact_length = N` — exact length
/// - `email` — valid email format
/// - `url` — valid URL format
/// - `uri` — valid URI format
/// - `ip` — valid IP address
/// - `hostname` — valid hostname
/// - `pattern = "regex"` — matches regex pattern
/// - `min = N` — minimum numeric value
/// - `max = N` — maximum numeric value
/// - `range(min = A, max = B)` — numeric range
/// - `step = N` — numeric step/divisibility
/// - `one_of = [a, b, c]` — value must be one of the listed values
/// - `custom = "path::to::fn"` — custom validation function
/// - `nested` — field implements Fieldset; delegate validation
/// - `message = "..."` — custom error message
/// - `message_fn = "path"` — dynamic message provider
/// - `locale = "en"` — locale for messages
///
/// ## Field-level filtering (`#[filter(...)]`)
///
/// - `trim` — trim whitespace
/// - `lowercase` — convert to lowercase
/// - `uppercase` — convert to uppercase
/// - `strip_tags` — remove HTML tags
/// - `html_entities` — encode HTML entities
/// - `slug` / `slug(max_length = N)` — slugify
/// - `truncate(max_length = N)` — truncate to length
/// - `replace(from = "x", to = "y")` — string replacement
/// - `clamp(min = A, max = B)` — clamp numeric value
/// - `custom = "path::to::fn"` — custom filter function
/// - `try_custom = "path::to::fn"` — fallible custom filter
/// - `nested` — field implements Fieldset; delegate filtering
#[proc_macro_derive(Fieldset, attributes(validate, filter, cross_validate, fieldset))]
pub fn derive_fieldset(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  match derive_fieldset_impl(input) {
    Ok(tokens) => tokens.into(),
    Err(e) => e.to_compile_error().into(),
  }
}

fn derive_fieldset_impl(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
  let struct_name = &input.ident;
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  // Only support named structs
  let fields = match &input.data {
    Data::Struct(ds) => match &ds.fields {
      Fields::Named(named) => &named.named,
      _ => {
        return Err(syn::Error::new_spanned(
          struct_name,
          "Fieldset can only be derived for structs with named fields",
        ));
      }
    },
    _ => {
      return Err(syn::Error::new_spanned(
        struct_name,
        "Fieldset can only be derived for structs",
      ));
    }
  };

  // Parse struct-level attributes
  let struct_attrs = parse_fieldset_struct_attrs(&input.attrs);
  let cross_validate = parse_cross_validate_attrs(&input.attrs);

  // Parse all fields
  let field_infos: Vec<_> = fields.iter().map(parse_field_info).collect();

  // Generate the const
  let break_on_failure = struct_attrs.break_on_failure;

  // Generate validate and filter methods
  let validate_fn = gen_validate(
    &field_infos,
    &cross_validate.fns,
    struct_attrs.break_on_failure,
  );
  let filter_fn = gen_filter(&field_infos);

  Ok(quote! {
    impl #impl_generics walrs_fieldfilter::Fieldset for #struct_name #ty_generics #where_clause {
      const BREAK_ON_FAILURE: bool = #break_on_failure;

      #validate_fn
      #filter_fn
    }
  })
}
