// extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
/// Attribute macro to enable logging a Result::Err variant if tagged
/// to a function that returns a Result enum, and it returns an Err.
/// Note: Returned E of Result<T, E> must implment the Display trait.
/// Also, crate must have the log crate as a dependency.
pub fn log_result_err(attr: TokenStream, item: TokenStream) -> TokenStream {
	let log_level = parse_macro_input!(attr as syn::Expr);
	let input = parse_macro_input!(item as syn::ItemFn);

	let log_errors_fn = generate_log_result_fn(&input, &log_level);

	let output = quote! {
		#log_errors_fn
	};

	output.into()
}

fn generate_log_result_fn(input: &syn::ItemFn, log_level: &syn::Expr) -> proc_macro2::TokenStream {
	let vis = &input.vis;
	let sig = &input.sig;
	let fn_name = &input.sig.ident;
	let context = fn_name.to_string();
	let block = &input.block;

	quote! {
		#vis #sig {
			(|| -> Result<_, _> #block)()
				.map_err(|err| {
					log::log!(#log_level, "{}: {}", #context, &err);
					err
				})
		}
	}
}
