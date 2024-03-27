use anyhow::{anyhow, bail};
use proc_macro::TokenStream;
use quote::quote;
use syn::{fold::Fold, parse::Parser, DeriveInput, MetaNameValue};

//holds the value we found from folding
//probably should visit instead of folding
#[derive(Default)]
struct SocmParser {
	const_name: Option<String>,
	ident: Option<syn::Ident>,
	visibility: Option<syn::Visibility>,
}

impl SocmParser {
	fn const_name_token(&self) -> Option<proc_macro2::TokenStream> {
		self.const_name
			.clone()
			.map(|string| string.parse().unwrap())
	}
}

impl Fold for SocmParser {
	fn fold_derive_input(&mut self, derive_input: syn::DeriveInput) -> syn::DeriveInput {
		if !derive_input.generics.params.is_empty() {
			panic!("size_of_const_macro: SizeOf derive macro does not yet support generics");
		}

		let ident = derive_input.ident.clone();

		self.visibility.get_or_insert(derive_input.vis.clone());
		self.ident.get_or_insert(ident.clone());

		if self.const_name.is_none() {
			let mut const_name = String::from("SIZE_OF");

			push_snake_case(&mut const_name, &ident.to_string());
			self.const_name.replace(const_name);
		}

		derive_input
	}
}

/// Get the value of a string literal in a MetaNameValue struct.
fn mnv_str_literal(meta_name_value: &MetaNameValue) -> Option<String> {
	match &meta_name_value.value {
		syn::Expr::Lit(expression) => {
			if let syn::Lit::Str(lit_str) = &expression.lit {
				return Some(lit_str.value());
			}
		}

		_ => {}
	}

	None
}

//conver FooBar -> FOO_BAR
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

/// If the Reuslt is of the Err variant, returns a compile_error!(..) token stream with the error message.
/// Otherwise, returns the value inside the Ok variant.  
/// This is not designed to panic on either variant.
fn unwrap_token_stream(token_result: anyhow::Result<TokenStream>) -> TokenStream {
	match token_result {
		Ok(stream) => stream,
		Err(error) => {
			let error_message = format!("size_of_const_macro: {error:#?}");

			let quoted = quote! {
				compile_error!(#error_message);
			};

			TokenStream::from(quoted)
		}
	}
}

fn parse_attribute(
	argument_tokens: TokenStream,
	item_tokens: TokenStream,
) -> anyhow::Result<TokenStream> {
	let Ok(arguments) = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
		.parse(argument_tokens)
	else {
		bail!("failed to parse punctuation");
	};

	let derive_input = syn::parse::<DeriveInput>(item_tokens)
		.map_err(|error| anyhow!("failed to parse input {error:#?}"))?;
	let mut socm_parser = SocmParser::default();
	let folded = socm_parser.fold_derive_input(derive_input);

	for argument in arguments {
		match argument {
			syn::Meta::Path(path) => {
				if path.get_ident().unwrap().to_string() == "private" {
					socm_parser.visibility = Some(syn::Visibility::Inherited);
				}
			}

			syn::Meta::NameValue(meta_name_value) => {
				let Some(value) = mnv_str_literal(&meta_name_value) else {
					bail!("size_of_const values must be a str literal");
				};

				match meta_name_value
					.path
					.get_ident()
					.unwrap()
					.to_string()
					.as_str()
				{
					"name" => {
						socm_parser.const_name = Some(value);
					}

					"visibility" => {
						socm_parser.visibility = Some(
							syn::parse_str::<syn::Visibility>(&value)
								.expect("size_of_const failed to parse visibility"),
						);
					}

					_ => {}
				}
			}

			syn::Meta::List(_) => panic!("size_of_const malformed attribute arguments"),
		}
	}

	let (Some(const_name), Some(ident)) = (socm_parser.const_name_token(), socm_parser.ident)
	else {
		bail!("failed to parse")
	};

	let quoted = if let Some(visibility) = socm_parser.visibility {
		quote! {
			#folded
			#visibility const #const_name: usize = ::core::mem::size_of::<#ident>();
		}
	} else {
		quote! {
			#folded
			const #const_name: usize = ::core::mem::size_of::<#ident>();
		}
	};

	Ok(TokenStream::from(quoted))
}

fn parse_derive(tokens: TokenStream) -> anyhow::Result<TokenStream> {
	let derive_input = syn::parse::<DeriveInput>(tokens)
		.map_err(|error| anyhow!("failed to parse input {error:#?}"))?;

	let mut socm_parser = SocmParser::default();
	let _ = socm_parser.fold_derive_input(derive_input);

	let (Some(const_name), Some(ident)) = (socm_parser.const_name_token(), socm_parser.ident)
	else {
		bail!("failed to parse")
	};

	let quoted = if let Some(visibility) = socm_parser.visibility {
		quote! {
			#visibility const #const_name: usize = ::core::mem::size_of::<#ident>();
		}
	} else {
		quote! {
			const #const_name: usize = ::core::mem::size_of::<#ident>();
		}
	};

	Ok(TokenStream::from(quoted))
}

/// Generates a constant of the type's size.
/// The constant copies the visibility of the applied type.
/// # Examples
/// Generated constant is named `SIZE_OF_FOO`, which inherits `Foo`'s (private) visibility.
/// This behaves the same as using `#[derive(SizeOf)]`.
/// ```
/// # use size_of_const_macro::size_of_const;
/// #[size_of_const]
/// struct Foo {
/// 	short: u16,
/// 	long: u32,
/// }
/// ```
/// Generated constant is public, and named `SIZE_OF_OTHER_FOO`.
/// ```
/// # use size_of_const_macro::size_of_const;
/// # #[size_of_const]
/// # struct Foo {
/// # 	short: u16,
/// # 	long: u32,
/// # }
/// #[size_of_const(visibility = "pub", name = "SIZE_OF_OTHER_FOO")]
/// struct Bar {
/// 	text: String,
/// 	foo: Foo,
/// }
/// ```
/// Generated constant is private, and named `SIZE_OF_BAZ_BIN`.
/// ```
/// # use size_of_const_macro::size_of_const;
/// #[size_of_const(private)]
/// pub enum BazBin {
/// 	Biz(String),
/// 	Boz(Vec<u8>),
/// }
/// ```
#[proc_macro_attribute]
pub fn size_of_const(argument_tokens: TokenStream, item_tokens: TokenStream) -> TokenStream {
	unwrap_token_stream(parse_attribute(argument_tokens, item_tokens))
}

/// Generates a constant of the type's size.
/// The constant copies the visibility of the applied type.
/// # Example
/// Generated constant is named `SIZE_OF_FOO_BAR`, which inherits `FooBar`'s (private) visibility.
/// ```
/// # use size_of_const_macro::SizeOf;
/// #[derive(SizeOf)]
/// struct FooBar {
/// 	short: u16,
/// 	long: u32,
/// }
/// ```
/// If you need to customize the name or visibility of the constant, use the `size_of_const` attribute instead.
#[proc_macro_derive(SizeOf)]
pub fn size_of_const_derive(tokens: TokenStream) -> TokenStream {
	unwrap_token_stream(parse_derive(tokens))
}
