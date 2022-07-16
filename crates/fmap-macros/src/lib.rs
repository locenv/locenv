use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, FnArg, GenericArgument, Ident, Lit, Meta,
    MetaList, NestedMeta, PathArguments, Receiver, Token, Type, VisPublic, Visibility,
};

#[proc_macro_derive(Directory, attributes(directory, file, placeholder))]
pub fn derive_directory(item: TokenStream) -> TokenStream {
    // Get list of field.
    let input = parse_macro_input!(item as DeriveInput);
    let data = if let Data::Struct(v) = &input.data {
        v
    } else {
        panic!("The Container can only apply on struct")
    };

    let fields = if let Fields::Named(v) = &data.fields {
        v
    } else {
        panic!("The Container can only apply on a structs with named fields")
    };

    // Iterate fields.
    let mut file_visibilities: Vec<Visibility> = Vec::new();
    let mut file_getters: Vec<&Ident> = Vec::new();
    let mut file_types: Vec<&Type> = Vec::new();
    let mut file_names: Vec<String> = Vec::new();

    let mut dir_visibilities: Vec<Visibility> = Vec::new();
    let mut dir_getters: Vec<&Ident> = Vec::new();
    let mut dir_args: Vec<FnArg> = Vec::new();
    let mut dir_types: Vec<&Type> = Vec::new();
    let mut dir_names: Vec<String> = Vec::new();

    let mut ph_visibilities: Vec<Visibility> = Vec::new();
    let mut ph_getters: Vec<&Ident> = Vec::new();
    let mut ph_names: Vec<String> = Vec::new();

    for field in &fields.named {
        let name = field.ident.as_ref().unwrap();

        // Parse attributes.
        let mut r#type: Option<DirectoryField> = None;

        for attr in &field.attrs {
            let meta = match attr.parse_meta() {
                Ok(r) => r,
                Err(_) => continue,
            };

            r#type = parse_directory_field(&meta);

            if r#type.is_some() {
                break;
            }
        }

        // Check type.
        let r#type = if let Some(v) = r#type {
            v
        } else {
            continue;
        };

        match r#type {
            DirectoryField::File(file) => {
                file_visibilities.push(if file.public {
                    Visibility::Public(VisPublic {
                        pub_token: <Token![pub]>::default(),
                    })
                } else {
                    Visibility::Inherited
                });

                file_getters.push(name);

                file_types.push(if let Type::Path(path) = &field.ty {
                    let last = path.path.segments.last().unwrap();
                    let args = if let PathArguments::AngleBracketed(v) = &last.arguments {
                        v
                    } else {
                        panic!("Field {} has invalid type", name)
                    };

                    if let GenericArgument::Type(v) = args.args.first().unwrap() {
                        v
                    } else {
                        panic!("Field {} has invalid type", name)
                    }
                } else {
                    panic!("Field {} has invalid type", name)
                });

                file_names.push(file.get_name(name));
            }
            DirectoryField::Directory(directory) => {
                dir_visibilities.push(if directory.public {
                    Visibility::Public(VisPublic {
                        pub_token: <Token![pub]>::default(),
                    })
                } else {
                    Visibility::Inherited
                });

                dir_getters.push(name);

                dir_args.push(FnArg::Receiver(if directory.borrow_parent {
                    Receiver {
                        attrs: Vec::new(),
                        reference: Some((<Token![&]>::default(), None)),
                        mutability: None,
                        self_token: <Token![self]>::default(),
                    }
                } else {
                    Receiver {
                        attrs: Vec::new(),
                        reference: None,
                        mutability: None,
                        self_token: <Token![self]>::default(),
                    }
                }));

                dir_types.push(if let Type::Path(path) = &field.ty {
                    let last = path.path.segments.last().unwrap();
                    let args = if let PathArguments::AngleBracketed(v) = &last.arguments {
                        v
                    } else {
                        panic!("Field {} has invalid type", name)
                    };

                    if let GenericArgument::Type(v) = args.args.first().unwrap() {
                        v
                    } else {
                        panic!("Field {} has invalid type", name)
                    }
                } else {
                    panic!("Field {} has invalid type", name)
                });

                dir_names.push(directory.get_name(name));
            }
            DirectoryField::Placeholder(ph) => {
                ph_visibilities.push(if ph.public {
                    Visibility::Public(VisPublic {
                        pub_token: <Token![pub]>::default(),
                    })
                } else {
                    Visibility::Inherited
                });

                ph_getters.push(name);
                ph_names.push(ph.get_name(name));
            }
        };
    }

    // Generate getters.
    let generics = input.generics;
    let ident = input.ident;
    let result = quote! {
        impl #generics #ident #generics {
            #( #file_visibilities fn #file_getters (&self) -> #file_types { <#file_types>::new(self.path(), #file_names) } )*
            #( #dir_visibilities fn #dir_getters (#dir_args) -> #dir_types { <#dir_types>::new(self, #dir_names) } )*
            #( #ph_visibilities fn #ph_getters (&self) -> std::path::PathBuf { let mut p = self.path(); p.push(#ph_names); p } )*
        }
    };

    result.into()
}

fn parse_directory_field(meta: &Meta) -> Option<DirectoryField> {
    let r#type = match meta {
        Meta::Path(p) => {
            if p.is_ident("file") {
                DirectoryField::File(File {
                    public: false,
                    name: None,
                    kebab: false,
                    extension: None,
                })
            } else if p.is_ident("directory") {
                DirectoryField::Directory(Directory {
                    public: false,
                    name: None,
                    kebab: false,
                    extension: None,
                    borrow_parent: false,
                })
            } else if p.is_ident("placeholder") {
                DirectoryField::Placeholder(Placeholder {
                    public: false,
                    name: None,
                    kebab: false,
                    extension: None,
                })
            } else {
                return None;
            }
        }
        Meta::List(l) => {
            if l.path.is_ident("file") {
                DirectoryField::File(File::parse(l))
            } else if l.path.is_ident("directory") {
                DirectoryField::Directory(Directory::parse(l))
            } else if l.path.is_ident("placeholder") {
                DirectoryField::Placeholder(Placeholder::parse(l))
            } else {
                return None;
            }
        }
        _ => return None,
    };

    Some(r#type)
}

enum DirectoryField {
    File(File),
    Directory(Directory),
    Placeholder(Placeholder),
}

struct File {
    public: bool,
    name: Option<String>,
    kebab: bool,
    extension: Option<String>,
}

impl File {
    fn parse(meta: &MetaList) -> Self {
        let mut public = false;
        let mut name: Option<String> = None;
        let mut kebab = false;
        let mut extension: Option<String> = None;

        for item in &meta.nested {
            if let Some(v) = parse_common_directory_field(item) {
                match v {
                    DirectoryFieldAttribute::Public => public = true,
                    DirectoryFieldAttribute::Name(v) => name = Some(v),
                    DirectoryFieldAttribute::Kebab => kebab = true,
                    DirectoryFieldAttribute::Extension(v) => extension = Some(v),
                }
            }
        }

        Self {
            name,
            public,
            kebab,
            extension,
        }
    }
}

impl Field for File {
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn kebab(&self) -> bool {
        self.kebab
    }

    fn extension(&self) -> Option<&str> {
        self.extension.as_deref()
    }
}

struct Directory {
    public: bool,
    name: Option<String>,
    kebab: bool,
    extension: Option<String>,
    borrow_parent: bool,
}

impl Directory {
    fn parse(meta: &MetaList) -> Self {
        let mut public = false;
        let mut name: Option<String> = None;
        let mut kebab = false;
        let mut extension: Option<String> = None;
        let mut borrow_parent = false;

        for item in &meta.nested {
            if let Some(v) = parse_common_directory_field(item) {
                match v {
                    DirectoryFieldAttribute::Public => public = true,
                    DirectoryFieldAttribute::Name(v) => name = Some(v),
                    DirectoryFieldAttribute::Kebab => kebab = true,
                    DirectoryFieldAttribute::Extension(v) => extension = Some(v),
                }
            } else if let NestedMeta::Meta(item) = item {
                if let Meta::Path(path) = item {
                    if path.is_ident("borrow_parent") {
                        borrow_parent = true;
                    }
                }
            }
        }

        Self {
            public,
            name,
            kebab,
            extension,
            borrow_parent,
        }
    }
}

impl Field for Directory {
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn kebab(&self) -> bool {
        self.kebab
    }

    fn extension(&self) -> Option<&str> {
        self.extension.as_deref()
    }
}

struct Placeholder {
    public: bool,
    name: Option<String>,
    kebab: bool,
    extension: Option<String>,
}

impl Placeholder {
    fn parse(meta: &MetaList) -> Self {
        let mut public = false;
        let mut name: Option<String> = None;
        let mut kebab = false;
        let mut extension: Option<String> = None;

        for item in &meta.nested {
            if let Some(v) = parse_common_directory_field(item) {
                match v {
                    DirectoryFieldAttribute::Public => public = true,
                    DirectoryFieldAttribute::Name(v) => name = Some(v),
                    DirectoryFieldAttribute::Kebab => kebab = true,
                    DirectoryFieldAttribute::Extension(v) => extension = Some(v),
                }
            }
        }

        Self {
            name,
            public,
            kebab,
            extension,
        }
    }
}

impl Field for Placeholder {
    fn name<'a>(&'a self) -> Option<&'a str> {
        self.name.as_deref()
    }

    fn kebab(&self) -> bool {
        self.kebab
    }

    fn extension(&self) -> Option<&str> {
        self.extension.as_deref()
    }
}

trait Field {
    fn name(&self) -> Option<&str>;
    fn kebab(&self) -> bool;
    fn extension(&self) -> Option<&str>;

    fn get_name(&self, ident: &Ident) -> String {
        let mut result: String = if let Some(name) = self.name() {
            name.into()
        } else {
            let mut name = ident.to_string();

            if self.kebab() {
                name = name.replace('_', "-");
            }

            name
        };

        if let Some(v) = self.extension() {
            result.push('.');
            result.push_str(v);
        }

        result
    }
}

fn parse_common_directory_field(meta: &NestedMeta) -> Option<DirectoryFieldAttribute> {
    let item = if let NestedMeta::Meta(v) = meta {
        v
    } else {
        return None;
    };

    match item {
        Meta::Path(p) => {
            if p.is_ident("pub") {
                return Some(DirectoryFieldAttribute::Public);
            } else if p.is_ident("kebab") {
                return Some(DirectoryFieldAttribute::Kebab);
            }
        }
        Meta::NameValue(p) => {
            if p.path.is_ident("name") {
                if let Lit::Str(v) = &p.lit {
                    return Some(DirectoryFieldAttribute::Name(v.value()));
                }
            } else if p.path.is_ident("ext") {
                if let Lit::Str(v) = &p.lit {
                    return Some(DirectoryFieldAttribute::Extension(v.value()));
                }
            }
        }
        _ => {}
    };

    None
}

enum DirectoryFieldAttribute {
    Public,
    Name(String),
    Kebab,
    Extension(String),
}
