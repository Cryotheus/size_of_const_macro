use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

fn push_snake_case(buffer: &mut String, camel_case: &str) {
	for char in camel_case.chars() {
		if char.is_uppercase() {
			buffer.push('_');
			buffer.push(char);
		} else {
			buffer.push_str(&char.to_uppercase().to_string());
		}
	}
}

/// Generates a constant that is the size of the derived type.
/// ```
///# use size_of_const_macro::SizeOf;
/// #[derive(SizeOf)]
/// struct FooBar {
/// 	short: u16,
/// 	long: u32,
/// }
///
/// assert_eq!(core::mem::size_of::<FooBar>(), SIZE_OF_FOO_BAR);
/// ```
#[proc_macro_derive(SizeOf)]
pub fn size_of_const_derive(tokens: TokenStream) -> TokenStream {
	let input = parse_macro_input!(tokens as DeriveInput);
	let ident = &input.ident;
	let ident_string = ident.to_string();
	let mut const_name = String::from("SIZE_OF");

	push_snake_case(&mut const_name, &ident_string);

	let const_name: proc_macro2::TokenStream = const_name.parse().unwrap();

	let quoted = quote! {
		const #const_name: usize = ::core::mem::size_of::<#ident>();
	};

	TokenStream::from(quoted)
}

#[proc_macro_attribute]
pub fn size_of(attr: TokenStream, item_tokens: TokenStream) -> TokenStream {
	let item_tokens_cloned = item_tokens.clone();
	let item_tokens: proc_macro2::TokenStream = proc_macro2::TokenStream::from(item_tokens);

	let input = parse_macro_input!(item_tokens_cloned as DeriveInput);
	let ident = &input.ident;
	let ident_string = ident.to_string();
	let mut const_name = String::from("SIZE_OF");

	push_snake_case(&mut const_name, &ident_string);

	let const_name: proc_macro2::TokenStream = const_name.parse().unwrap();

	let quoted = quote! {
		#item_tokens;
		const #const_name: usize = ::core::mem::size_of::<#ident>();
	};

	TokenStream::from(quoted)
}

#[proc_macro_attribute]
pub fn private_size_of(attr: TokenStream, item_tokens: TokenStream) -> TokenStream {
	let item_tokens_cloned = item_tokens.clone();
	let item_tokens: proc_macro2::TokenStream = proc_macro2::TokenStream::from(item_tokens);

	let input = parse_macro_input!(item_tokens_cloned as DeriveInput);
	let ident = &input.ident;
	let ident_string = ident.to_string();
	let mut const_name = String::from("SIZE_OF");

	push_snake_case(&mut const_name, &ident_string);

	let const_name: proc_macro2::TokenStream = const_name.parse().unwrap();

	let quoted = quote! {
		#item_tokens;
		const #const_name: usize = ::core::mem::size_of::<#ident>();
	};

	TokenStream::from(quoted)
}
