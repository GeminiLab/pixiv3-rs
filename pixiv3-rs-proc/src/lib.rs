#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::fold::Fold;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Expr, Ident, LitStr, Token, Type, TypeReference, braced, parenthesized, token,
};

/// One param or data field: `name @ "key": Type = default => transmute` (all optional after :)
struct ParamSpec {
    name: Ident,
    key_override: Option<LitStr>,
    ty: Type,
    default: Option<Expr>,
    transmute: Option<Expr>,
}

impl Parse for ParamSpec {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let key_override = if input.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            let lit: LitStr = input.parse()?;
            Some(lit)
        } else {
            None
        };
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;
        let default = if input.peek(Token![=]) && !input.peek2(Token![>]) {
            input.parse::<Token![=]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        let transmute = if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(ParamSpec {
            name,
            key_override,
            ty,
            default,
            transmute,
        })
    }
}

/// Optional "params [ ... ]" or "data [ ... ]" section
struct ParamSection {
    kind: Ident,
    entries: Vec<ParamSpec>,
}

impl Parse for ParamSection {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let kind: Ident = input.parse()?;
        let content;
        syn::bracketed!(content in input);
        let entries = content.parse_terminated(ParamSpec::parse, Token![,])?;
        Ok(ParamSection {
            kind,
            entries: entries.into_iter().collect(),
        })
    }
}

/// Whether the endpoint returns a paged result, and generates an iterator
/// function for the items.
struct Paged {
    #[cfg_attr(not(feature = "stream"), expect(dead_code))]
    field: Ident,
    #[cfg_attr(not(feature = "stream"), expect(dead_code))]
    item_type: Type,
    #[cfg_attr(not(feature = "stream"), expect(dead_code))]
    next_url: Option<Ident>,
}

impl Parse for Paged {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let paged = input.parse::<Ident>()?;
        if paged != "paged" {
            return Err(syn::Error::new(paged.span(), "expected paged"));
        }

        let next_url = if input.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            let next_url = input.parse::<Ident>()?;
            Some(next_url)
        } else {
            None
        };

        let field: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let item_type: Type = input.parse()?;
        Ok(Paged {
            field,
            item_type,
            next_url,
        })
    }
}

/// Full endpoint: attrs? name -> ReturnType (paged @next_url? field: ItemType)? { METHOD "url", params? data? }
struct ApiEndpoint {
    attrs: Vec<Attribute>,
    name: Ident,
    return_type: Type,
    #[cfg_attr(not(feature = "stream"), expect(dead_code))]
    paged: Option<Paged>,
    method: Ident,
    url: LitStr,
    sections: Vec<ParamSection>,
}

impl Parse for ApiEndpoint {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let name: Ident = input.parse()?;
        input.parse::<Token![->]>()?;
        let return_type: Type = input.parse()?;

        let paged = if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            let paged = content.parse::<Paged>()?;
            Some(paged)
        } else {
            None
        };

        let content;
        braced!(content in input);
        let method: Ident = content.parse()?;
        let url: LitStr = content.parse()?;
        content.parse::<Token![,]>()?;

        let mut sections = Vec::new();
        while !content.is_empty() {
            let lookahead = content.lookahead1();
            if lookahead.peek(Ident) {
                let section = content.parse::<ParamSection>()?;
                sections.push(section);
                if !content.is_empty() {
                    content.parse::<Token![,]>()?;
                }
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(ApiEndpoint {
            attrs,
            name,
            return_type,
            method,
            url,
            sections,
            paged,
        })
    }
}

impl ApiEndpoint {
    pub fn find_section(&self, kind: impl AsRef<str>) -> Option<&ParamSection> {
        let kind = kind.as_ref();
        self.sections.iter().find(|s| s.kind == kind)
    }
}

/// Fold that adds explicit lifetimes ('a1, 'a2, ...) to all references in a type,
/// and records them in the lifetimes vec.
struct ExplicitLifetimeFolder {
    counter: u32,
    lifetimes: Vec<syn::Lifetime>,
}

impl ExplicitLifetimeFolder {
    fn new() -> Self {
        Self {
            counter: 0,
            lifetimes: Vec::new(),
        }
    }
}

impl Fold for ExplicitLifetimeFolder {
    fn fold_type_reference(&mut self, mut ty_ref: TypeReference) -> TypeReference {
        if let Some(lifetime) = &ty_ref.lifetime {
            self.lifetimes.push(lifetime.clone());
        } else {
            self.counter += 1;
            let lt = syn::Lifetime::new(&format!("'a{}", self.counter), Span::call_site());
            self.lifetimes.push(lt.clone());
            ty_ref.lifetime = Some(lt);
            *ty_ref.elem = self.fold_type(*ty_ref.elem);
        }

        ty_ref
    }
}

struct ApiEndpoints {
    endpoints: Punctuated<ApiEndpoint, Token![;]>,
}

impl Parse for ApiEndpoints {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let endpoints = input.parse_terminated(ApiEndpoint::parse, Token![;])?;
        Ok(ApiEndpoints { endpoints })
    }
}

/// Generates async API methods on `AppPixivAPI` from endpoint definitions.
///
/// Syntax: one or more endpoints separated by `;`. Each endpoint:
/// `/// doc? name -> ReturnType (paged @next_url? field: ItemType)? { GET|POST|DELETE "path", params [ ... ]? data [ ... ]? }`
///
/// - Params: `name: Type = default => transmute`; use `name @ "key": Type` to override query/form key.
/// - Paged: `(paged illusts: IllustrationInfo)` generates a method returning a struct with `illusts` and `next_url`.
///
/// 根据端点定义在 `AppPixivAPI` 上生成异步 API 方法。语法：多个端点用 `;` 分隔；每条可含 doc、返回类型、可选 paged、方法、路径及 params/data。
#[proc_macro]
pub fn api_endpoints(input: TokenStream) -> TokenStream {
    let endpoints = match syn::parse::<ApiEndpoints>(input) {
        Ok(e) => e,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut expanded = TokenStream2::new();

    for endpoint in endpoints.endpoints {
        let attrs = &endpoint.attrs;
        let name = &endpoint.name;
        let return_type = &endpoint.return_type;
        let method = &endpoint.method;
        let url = &endpoint.url;

        let mut fn_params = Vec::new();
        let mut section_inits = Vec::new();
        let mut section_bodies = Vec::new();
        #[cfg(feature = "stream")]
        let mut fn_args = Vec::new();
        let mut folder = ExplicitLifetimeFolder::new();

        for section in &endpoint.sections {
            let kind = &section.kind;

            for spec in &section.entries {
                let name = &spec.name;
                let ty = folder.fold_type(spec.ty.clone());

                fn_params.push(quote! { #name: #ty, });

                let key = if let Some(key) = &spec.key_override {
                    quote! { #key }
                } else {
                    quote! { stringify!(#name) }
                };

                let mut body_for_this = TokenStream2::new();

                if let Some(default) = &spec.default {
                    body_for_this.extend(quote! {
                        let #name = #name.unwrap_or_else(|| #default);
                    });
                }

                if let Some(transmute) = &spec.transmute {
                    body_for_this.extend(quote! {
                        let #name = #transmute;
                    });
                }

                body_for_this.extend(quote! {
                    #kind.push(#key, #name);
                });

                section_bodies.push(quote! { { #body_for_this } });

                #[cfg(feature = "stream")]
                fn_args.push(quote! { #name, });
            }

            section_inits.push(quote! {
                #[allow(unused_mut)]
                let mut #kind: kv_pairs::KVPairs<'_> = kv_pairs::kv_pairs![];
            });
        }

        let params = if endpoint.find_section("params").is_some() {
            quote! { Some(params) }
        } else {
            quote! { None }
        };
        let data = if endpoint.find_section("data").is_some() {
            quote! { Some(data) }
        } else {
            quote! { None }
        };

        let lifetimes = &folder.lifetimes;
        let expanded_endpoint = quote! {
            #(#attrs)*
            #[allow(clippy::too_many_arguments)]
            pub async fn #name<'a0 #(, #lifetimes)*>(
                &'a0 self,
                #(#fn_params)*
                with_auth: bool,
            ) -> Result<#return_type, crate::error::PixivError> {
                let url = format!("{}{}", self.hosts, #url);
                #(#section_inits)*
                #(#section_bodies)*
                crate::debug!("calling {} at {}", stringify!(#name), #url);
                let r = self.do_api_request(crate::aapi::HttpMethod::#method, &url, None, #params, #data, with_auth).await?;
                crate::models::parse_response_into::<#return_type>(r).await
            }
        };

        expanded.extend(expanded_endpoint);

        #[cfg(feature = "stream")]
        if let Some(paged) = &endpoint.paged {
            use quote::format_ident;

            let iter_fn_name = format_ident!("{}_iter", name);
            let item_field = &paged.field;
            let item_type = &paged.item_type;
            let next_url_field = paged
                .next_url
                .clone()
                .unwrap_or_else(|| format_ident!("next_url"));
            let iter_doc_comment = format!(
                "Iterate over the results of {0}.\n\n{0}的迭代版本。",
                stringify!(#name)
            );

            let iter_fn = quote! {
                #[allow(clippy::too_many_arguments)]
                #[doc = #iter_doc_comment]
                pub fn #iter_fn_name<'a0 #(, #lifetimes)*>(
                    &'a0 self,
                    #(#fn_params)*
                    with_auth: bool,
                ) -> impl ::futures_core::stream::Stream<
                    Item = Result<#item_type, crate::error::PixivError>
                > + use<'a0 #(, #lifetimes)*> {
                    crate::debug!("calling {} (iterable version of {})", stringify!(#name), stringify!(#iter_fn_name));

                    async_stream::try_stream! {
                        crate::debug!("{} first request to {}", stringify!(#iter_fn_name), #url);
                        let mut result = self.#name(#(#fn_args)* with_auth).await?;
                        let mut next_url = result.#next_url_field;

                        loop {
                            for item in result.#item_field {
                                yield item;
                            }

                            match &next_url {
                                Some(url) => {
                                    crate::debug!("{} next request to {}", stringify!(#iter_fn_name), url);
                                    result = self.visit_next_url::<#return_type>(url, with_auth).await?;
                                    next_url = result.#next_url_field;
                                }
                                None => {
                                    crate::debug!("{} reached end of results", stringify!(#iter_fn_name));
                                    break;
                                },
                            }
                        }
                    }
                }
            };

            expanded.extend(iter_fn);
        }
    }

    TokenStream::from(expanded)
}

/// A no-op macro that does nothing. Used for placeholder or conditional compilation.
#[proc_macro]
pub fn no_op_macro(_: TokenStream) -> TokenStream {
    TokenStream::new()
}
