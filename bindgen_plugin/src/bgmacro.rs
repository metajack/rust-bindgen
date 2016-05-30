use std::default::Default;
use std::fmt;
use std::path::Path;

use syntax::ast::{Ident, Lit};
use syntax::ast::LitKind::*;
use syntax::codemap::{self, Span, Spanned};
use syntax::ext::base;
use syntax::parse;
use syntax::util::small_vector::SmallVector;

use bindgen::{Builder, LinkType, Logger};

use clang_sys::support::Clang;

use parser;

use easy_plugin::PluginResult;

easy_plugin! {
    struct Arguments {
        $($file:lit), *
        $(, $($key:ident = $value:lit), *)?
    }

    pub fn bindgen_macro(cx: &mut base::ExtCtxt,
                         sp: Span,
                         args: Arguments)
                         -> PluginResult<Box<base::MacResult + 'static>> {
        fn args_to_options(args: Arguments) -> PluginResult<Builder<'static>> {
            let mut builder = Builder::default();

            for (key, value) in args.key.unwrap().iter().zip(args.value.unwrap()) {
               try!(decode_key_value(&key, &value, &mut builder));
            }

            Ok(builder)
        }

        let logger = MacroLogger {
            // We want the span for errors to just match the bindgen! symbol
            // instead of the whole invocation which can span multiple lines
            sp: Span {
                hi: sp.lo + codemap::BytePos(8),
                ..sp
            },
            cx: cx,
        };
        let mut builder = try!(args_to_options(args));
        builder.log(&logger);

        let clang = Clang::find(None).expect("No clang found, is it installed?");
        for dir in clang.c_search_paths {
            builder.clang_arg("-idirafter");
            builder.clang_arg(dir.to_str().unwrap());
        }

        // Add the directory of the header to the include search path.
        let filename = cx.codemap().span_to_filename(sp);
        let mod_dir = Path::new(&filename).parent().unwrap();
        builder.clang_arg("-I");
        builder.clang_arg(mod_dir.to_str().unwrap());

        match builder.generate() {
            Ok(bindings) => {
                // syntex_syntax is not compatible with libsyntax so convert to string and reparse
                let bindings_str = bindings.to_string();
                // Unfortunately we lose span information due to reparsing
                let mut parser = parse::new_parser_from_source_str(cx.parse_sess(),
                                                                   cx.cfg(),
                                                                   "(Auto-generated bindings)"
                                                                       .to_string(),
                                                                   bindings_str);

                let mut items = Vec::new();
                while let Ok(Some(item)) = parser.parse_item() {
                    items.push(item);
                }

                Ok(Box::new(base::MacEager {
                    items: Some(SmallVector::many(items)),
                    ..Default::default()
                }))

            }
            Err(()) => Err((sp, "Don't work".into())),
        }
    }
}

fn decode_key_value(key: &Spanned<Ident>, value: &Lit, builder: &mut Builder) -> PluginResult<()> {
    let key_str: &str = &key.node.name.as_str();
    match (key_str, &value.node) {
        ("builtins", &Bool(b)) => {
            if b {
                builder.builtins();
            }
        }
        ("match", &Str(ref s, _)) => {
            builder.match_pat(s as &str);
        }
        ("allow_unknown_types", &Bool(b)) => {
            if !b {
                builder.forbid_unknown_types();
            }
        }
        ("clang_args", &Str(ref s, _)) => {
            for arg in parser::parse_process_args(s) {
                builder.clang_arg(arg);
            }
        }
        ("override_enum_type", &Str(ref s, _)) => {
            builder.override_enum_ty(s as &str);
        }
        ("link", &Str(ref s, _)) => {
            // TODO: dupplicate of main
            let parts = s.split('=').collect::<Vec<_>>();
            let (name, kind) = match parts.len() {
                1 => (parts[0], LinkType::Dynamic),
                2 => {
                    (parts[1],
                     match parts[0] {
                        "static" => LinkType::Static,
                        "dynamic" => LinkType::Dynamic,
                        "framework" => LinkType::Framework,
                        _ => return Err((value.span, "Invalid link type".into())),
                    })
                }
                _ => return Err((value.span, "Invalid link directive".into())),
            };
            builder.link(name, kind);
        }
        _ => return Err((key.span, format!("Invalid key or value: {}", key_str))),
    }
    Ok(())
}

struct MacroLogger<'a, 'b: 'a> {
    sp: codemap::Span,
    cx: &'a base::ExtCtxt<'b>,
}

impl<'a, 'b> Logger for MacroLogger<'a, 'b> {
    fn error(&self, msg: &str) {
        self.cx.span_err(self.sp, msg)
    }

    fn warn(&self, msg: &str) {
        self.cx.span_warn(self.sp, msg)
    }
}

// ExtCtxt is not Debug
impl<'a, 'b> fmt::Debug for MacroLogger<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MacroLogger")
    }
}
