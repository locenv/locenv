use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{
    Field, FnArg, GenericArgument, Ident, Lit, Meta, MetaList, NestedMeta, Pat, PatIdent, PatType,
    Path, PathArguments, PathSegment, Receiver, Token, Type, TypePath, VisPublic, Visibility,
};

pub struct DirectoryParser<'input> {
    file_visibilities: Vec<Visibility>,
    file_getters: Vec<&'input Ident>,
    file_types: Vec<&'input Type>,
    file_names: Vec<String>,
    dir_visibilities: Vec<Visibility>,
    dir_getters: Vec<&'input Ident>,
    dir_args: Vec<Punctuated<FnArg, Token![,]>>,
    dir_types: Vec<&'input Type>,
    dir_creates: Vec<TokenStream>,
    dir_names: Vec<String>,
    ph_visibilities: Vec<Visibility>,
    ph_getters: Vec<&'input Ident>,
    ph_names: Vec<String>,
}

impl<'input> DirectoryParser<'input> {
    pub fn new() -> Self {
        Self {
            file_visibilities: Vec::new(),
            file_getters: Vec::new(),
            file_types: Vec::new(),
            file_names: Vec::new(),
            dir_visibilities: Vec::new(),
            dir_getters: Vec::new(),
            dir_args: Vec::new(),
            dir_types: Vec::new(),
            dir_creates: Vec::new(),
            dir_names: Vec::new(),
            ph_visibilities: Vec::new(),
            ph_getters: Vec::new(),
            ph_names: Vec::new(),
        }
    }

    pub fn parse_field(&mut self, field: &'input Field) {
        let name = field.ident.as_ref().unwrap();

        // Parse attributes.
        let mut ty: Option<FieldType> = None;

        for attr in &field.attrs {
            let meta = match attr.parse_meta() {
                Ok(r) => r,
                Err(_) => continue,
            };

            ty = FieldType::parse(&meta);

            if ty.is_some() {
                break;
            }
        }

        // Check type.
        let ty = if let Some(v) = ty {
            v
        } else {
            return;
        };

        match ty {
            FieldType::File(data) => self.parse_file(name, &field.ty, &data),
            FieldType::Directory(data) => self.parse_directory(name, &field.ty, &data),
            FieldType::Placeholder(data) => self.parse_placeholder(name, &data),
        };
    }

    pub fn generate_body(&self) -> TokenStream {
        let fvisibilities = &self.file_visibilities;
        let fgetters = &self.file_getters;
        let ftypes = &self.file_types;
        let fnames = &self.file_names;
        let dvisibilities = &self.dir_visibilities;
        let dgetters = &self.dir_getters;
        let dargs = &self.dir_args;
        let dtypes = &self.dir_types;
        let dcreates = &self.dir_creates;
        let dnames = &self.dir_names;
        let pvisibilities = &self.ph_visibilities;
        let pgetters = &self.ph_getters;
        let pnames = &self.ph_names;

        quote! {
            #( #fvisibilities fn #fgetters (&self) -> #ftypes { <#ftypes>::new(self.path(), #fnames) } )*
            #( #dvisibilities fn #dgetters (#dargs) -> Result<#dtypes, dirtree::DirectoryError> {
                let dir = <#dtypes>::new(self, #dnames);
                if create {
                    #dcreates
                }
                Ok(dir)
            })*
            #( #pvisibilities fn #pgetters (&self) -> std::path::PathBuf {
                let mut p = self.path();
                p.push(#pnames);
                p
            })*
        }
    }

    fn parse_file(&mut self, name: &'input Ident, r#type: &'input Type, data: &File) {
        self.file_visibilities.push(if data.public {
            Visibility::Public(VisPublic {
                pub_token: <Token![pub]>::default(),
            })
        } else {
            Visibility::Inherited
        });

        self.file_getters.push(name);

        self.file_types.push(if let Type::Path(path) = r#type {
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

        self.file_names.push(data.get_name(name));
    }

    fn parse_directory(&mut self, name: &'input Ident, r#type: &'input Type, data: &Directory) {
        self.dir_visibilities.push(if data.public {
            Visibility::Public(VisPublic {
                pub_token: <Token![pub]>::default(),
            })
        } else {
            Visibility::Inherited
        });

        self.dir_getters.push(name);

        let mut args = Punctuated::new();

        args.push(FnArg::Receiver(if data.borrow_parent {
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

        args.push(FnArg::Typed(Self::create_parameter(
            PatIdent {
                attrs: Vec::new(),
                by_ref: None,
                mutability: None,
                ident: format_ident!("create"),
                subpat: None,
            },
            Type::Path(TypePath {
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments: Punctuated::from_iter(vec![PathSegment {
                        ident: format_ident!("bool"),
                        arguments: PathArguments::None,
                    }]),
                },
            }),
        )));

        self.dir_args.push(args);

        self.dir_types.push(if let Type::Path(path) = r#type {
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

        self.dir_creates.push(quote! {
            let path = dir.path();
            let created = match std::fs::create_dir(&path) {
                Ok(_) => true,
                Err(e) => match e.kind() {
                    std::io::ErrorKind::AlreadyExists => false,
                    _ => return Err(dirtree::DirectoryError::CreateFailed(e)),
                }
            };

            if created {}
        });

        self.dir_names.push(data.get_name(name));
    }

    fn parse_placeholder(&mut self, name: &'input Ident, data: &Placeholder) {
        self.ph_visibilities.push(if data.public {
            Visibility::Public(VisPublic {
                pub_token: <Token![pub]>::default(),
            })
        } else {
            Visibility::Inherited
        });

        self.ph_getters.push(name);
        self.ph_names.push(data.get_name(name));
    }

    fn create_parameter(name: PatIdent, r#type: Type) -> PatType {
        PatType {
            attrs: Vec::new(),
            pat: Box::new(Pat::Ident(name)),
            colon_token: <Token![:]>::default(),
            ty: Box::new(r#type),
        }
    }
}

enum FieldType {
    File(File),
    Directory(Directory),
    Placeholder(Placeholder),
}

impl FieldType {
    fn parse(attr: &Meta) -> Option<Self> {
        let ty = match attr {
            Meta::Path(p) => {
                if Self::is_file(p) {
                    Self::File(File::default())
                } else if Self::is_directory(p) {
                    Self::Directory(Directory::default())
                } else if Self::is_placeholder(p) {
                    Self::Placeholder(Placeholder::default())
                } else {
                    return None;
                }
            }
            Meta::List(l) => {
                if Self::is_file(&l.path) {
                    Self::File(File::parse(l))
                } else if Self::is_directory(&l.path) {
                    Self::Directory(Directory::parse(l))
                } else if Self::is_placeholder(&l.path) {
                    Self::Placeholder(Placeholder::parse(l))
                } else {
                    return None;
                }
            }
            _ => return None,
        };

        Some(ty)
    }

    fn is_file(p: &Path) -> bool {
        p.is_ident("file")
    }

    fn is_directory(p: &Path) -> bool {
        p.is_ident("directory")
    }

    fn is_placeholder(p: &Path) -> bool {
        p.is_ident("placeholder")
    }
}

#[derive(Default)]
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
            if let Some(v) = CommonAttribute::parse(item) {
                match v {
                    CommonAttribute::Public => public = true,
                    CommonAttribute::Name(v) => name = Some(v),
                    CommonAttribute::Kebab => kebab = true,
                    CommonAttribute::Extension(v) => extension = Some(v),
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

impl FieldData for File {
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

#[derive(Default)]
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
            if let Some(v) = CommonAttribute::parse(item) {
                match v {
                    CommonAttribute::Public => public = true,
                    CommonAttribute::Name(v) => name = Some(v),
                    CommonAttribute::Kebab => kebab = true,
                    CommonAttribute::Extension(v) => extension = Some(v),
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

impl FieldData for Directory {
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

#[derive(Default)]
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
            if let Some(v) = CommonAttribute::parse(item) {
                match v {
                    CommonAttribute::Public => public = true,
                    CommonAttribute::Name(v) => name = Some(v),
                    CommonAttribute::Kebab => kebab = true,
                    CommonAttribute::Extension(v) => extension = Some(v),
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

impl FieldData for Placeholder {
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

enum CommonAttribute {
    Public,
    Name(String),
    Kebab,
    Extension(String),
}

impl CommonAttribute {
    fn parse(meta: &NestedMeta) -> Option<Self> {
        let item = if let NestedMeta::Meta(v) = meta {
            v
        } else {
            return None;
        };

        match item {
            Meta::Path(p) => {
                if p.is_ident("pub") {
                    return Some(Self::Public);
                } else if p.is_ident("kebab") {
                    return Some(Self::Kebab);
                }
            }
            Meta::NameValue(p) => {
                if p.path.is_ident("name") {
                    if let Lit::Str(v) = &p.lit {
                        return Some(Self::Name(v.value()));
                    }
                } else if p.path.is_ident("ext") {
                    if let Lit::Str(v) = &p.lit {
                        return Some(Self::Extension(v.value()));
                    }
                }
            }
            _ => {}
        };

        None
    }
}

trait FieldData {
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
