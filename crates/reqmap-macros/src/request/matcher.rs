use super::pattern::Segment;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{Ident, Token, TypePath};

pub struct MatcherBuilder<'variant, 'segments, 'params> {
    variant: &'variant Ident,
    segments: &'segments [Segment<'segments>],
    params: &'params [&'params TypePath],
}

impl<'variant, 'segments, 'params> MatcherBuilder<'variant, 'segments, 'params> {
    pub fn new(
        variant: &'variant Ident,
        segments: &'segments [Segment<'segments>],
        params: &'params [&'params TypePath],
    ) -> Self {
        Self {
            variant,
            segments,
            params,
        }
    }

    pub fn build(&self) -> Result<TokenStream, BuilderError> {
        self.next(0, 0)
    }

    fn next(&self, segment: usize, param: usize) -> Result<TokenStream, BuilderError> {
        // Check if all segments have been matched.
        let block = if segment == self.segments.len() {
            // Check if variant have parameters.
            let variant = self.variant;

            if self.params.is_empty() {
                quote! {
                    return Some(Self::#variant);
                }
            } else {
                // Build parameter names.
                let mut params = Punctuated::<Ident, Token![,]>::new();

                for i in 0..self.params.len() {
                    params.push(Self::build_parameter_name(i));
                }

                quote! {
                    return Some(Self::#variant(#params));
                }
            }
        } else {
            // Check segment type.
            match self.segments[segment] {
                Segment::Static(value) => {
                    let next = self.next(segment + 1, param)?;

                    quote! {
                        if let Ok(decoded) = percent_encoding::percent_decode_str(segments[#segment]).decode_utf8() {
                            if decoded.as_ref() == #value {
                                #next
                            }
                        }
                    }
                }
                Segment::Param => {
                    // Check if variant have corresponding field.
                    if param == self.params.len() {
                        return Err(BuilderError::PatternAndParametersMismatched);
                    }

                    // Build matcher.
                    let name = Self::build_parameter_name(param);
                    let ty = self.params[param];
                    let next = self.next(segment + 1, param + 1)?;

                    quote! {
                        if let Ok(decoded) = percent_encoding::percent_decode_str(segments[#segment]).decode_utf8() {
                            let segment = decoded.as_ref();

                            if !segment.is_empty() {
                                if let Ok(#name) = segment.parse::<#ty>() {
                                    #next
                                }
                            }
                        }
                    }
                }
            }
        };

        Ok(block)
    }

    fn build_parameter_name(index: usize) -> Ident {
        format_ident!("param{}", index)
    }
}

pub enum BuilderError {
    PatternAndParametersMismatched,
}
