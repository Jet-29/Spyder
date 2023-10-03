use proc_macro::TokenStream;
use quote::quote;
use std::cell::RefCell;
use std::path::Path;
use std::{env, fs};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Ident, LitStr, Token};

#[proc_macro]
pub fn include_glsl(glsl_options: TokenStream) -> TokenStream {
    let Spirv { bytes, sources } = parse_macro_input!(glsl_options);
    quote!(
        {
            #({const _FORCE_INCLUDE: &[u8] = include_bytes!(#sources);})*
            &[#(#bytes),*]
        }
    )
    .into()
}

struct Spirv {
    bytes: Vec<u32>,
    sources: Vec<String>,
}

impl Parse for Spirv {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path_lit = input.parse::<LitStr>()?;
        let path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join(path_lit.value());
        let path_str = path.to_string_lossy().into_owned();

        let src = fs::read_to_string(&path).map_err(|e| syn::Error::new(path_lit.span(), e))?;
        let sources = RefCell::new(vec![path_str.clone()]);

        let build_options = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?; // skip comma
            input.parse::<CompileOptions>()?
        } else {
            CompileOptions::default()
        };

        let mut shaderc_options = shaderc::CompileOptions::new().unwrap();
        shaderc_options.set_warnings_as_errors();
        shaderc_options.set_include_callback(|name, include_type, src, _depth| {
            let path = match include_type {
                shaderc::IncludeType::Relative => Path::new(src).parent().unwrap().join(name),
                shaderc::IncludeType::Standard => {
                    Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join(name)
                }
            };
            let path_str = path.to_str().ok_or("non-unicode path")?.to_owned();
            sources.borrow_mut().push(path_str.clone());
            Ok(shaderc::ResolvedInclude {
                resolved_name: path_str,
                content: fs::read_to_string(path).map_err(|x| x.to_string())?,
            })
        });

        for (name, value) in build_options.definitions {
            shaderc_options.add_macro_definition(&name, value.as_deref());
        }

        shaderc_options.set_optimization_level(build_options.optimization);
        let kind = build_options
            .kind
            .or_else(|| {
                path.extension()
                    .and_then(|ext| str_to_kind(ext.to_str().unwrap()))
            })
            .unwrap_or(shaderc::ShaderKind::InferFromSource);

        let compiler = shaderc::Compiler::new().unwrap();
        let artifact = compiler
            .compile_into_spirv(
                &src,
                kind,
                path_str.as_str(),
                "main",
                Some(&shaderc_options),
            )
            .map_err(|e| {
                let mut msg = format!("{}\n", e);
                for src in sources.borrow().iter() {
                    msg += &format!("included from {}\n", src);
                }
                syn::Error::new(path_lit.span(), msg)
            })?;

        drop(shaderc_options);

        Ok(Self {
            bytes: artifact.as_binary().to_vec(),
            sources: sources.into_inner(),
        })
    }
}

struct CompileOptions {
    kind: Option<shaderc::ShaderKind>,
    definitions: Vec<(String, Option<String>)>,
    optimization: shaderc::OptimizationLevel,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            kind: None,
            definitions: Vec::new(),
            optimization: shaderc::OptimizationLevel::Performance,
        }
    }
}

impl Parse for CompileOptions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut kind: Option<shaderc::ShaderKind> = None;
        let mut definitions = Vec::new();
        let mut optimization = shaderc::OptimizationLevel::Performance;

        while input.peek(Ident) {
            let key = input.parse::<Ident>()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "kind" => {
                    let value = input.parse::<Ident>()?;
                    if let Some(new_kind) = str_to_kind(&value.to_string()) {
                        kind = Some(new_kind);
                    } else {
                        return Err(syn::Error::new(value.span(), "unknown shader kind"));
                    }
                }

                "define" => {
                    let name = input.parse::<Ident>()?.to_string();
                    let value = if input.peek(Token![=]) {
                        input.parse::<Token![=]>()?;
                        Some(input.parse::<LitStr>()?.value())
                    } else {
                        None
                    };
                    definitions.push((name, value));
                }

                "optimize" => {
                    let value = input.parse::<Ident>()?;
                    optimization = match value.to_string().as_str() {
                        "none" => shaderc::OptimizationLevel::Zero,
                        "size" => shaderc::OptimizationLevel::Size,
                        "performance" => shaderc::OptimizationLevel::Performance,
                        _ => {
                            return Err(syn::Error::new(value.span(), "unknown optimization level"))
                        }
                    };
                }
                _ => return Err(syn::Error::new(key.span(), "unknown option")),
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                break;
            }
        }

        Ok(Self {
            kind,
            definitions,
            optimization,
        })
    }
}

fn str_to_kind(s: &str) -> Option<shaderc::ShaderKind> {
    use shaderc::ShaderKind::*;
    Some(match s {
        "vert" => Vertex,
        "frag" => Fragment,
        "comp" => Compute,
        "geom" => Geometry,
        "tesc" => TessControl,
        "tese" => TessEvaluation,
        "spvasm" => SpirvAssembly,
        "rgen" => RayGeneration,
        "rahit" => AnyHit,
        "rchit" => ClosestHit,
        "rmiss" => Miss,
        "rint" => Intersection,
        "rcall" => Callable,
        "task" => Task,
        "mesh" => Mesh,
        _ => return None,
    })
}
