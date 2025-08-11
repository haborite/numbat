use itertools::Itertools;
use numbat::command::{CommandControlFlow, CommandRunner};
use numbat::module_importer::{BuiltinModuleImporter, ChainedImporter, FileSystemImporter};
use numbat::pretty_print::PrettyPrint;
use numbat::resolver::CodeSource;
use numbat::{Context, NumbatError};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
// use std::{env, fs, thread};
// use std::rc::{Rc, Weak};
// use std::cell::RefCell;

use dioxus::prelude::*;


#[derive(Debug, PartialEq, Eq)]
pub enum ExitStatus {
    Success,
    Error,
}


fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    config_dir.join("numbat")
}

fn get_modules_paths() -> Vec<PathBuf> {
    let mut paths = vec![];
    if let Some(modules_path) = std::env::var_os("NUMBAT_MODULES_PATH") {
        for path in modules_path.to_string_lossy().split(':') {
            paths.push(path.into());
        }
    }
    paths.push(get_config_path().join("modules"));

    if let Some(system_module_path) = option_env!("NUMBAT_SYSTEM_MODULE_PATH") {
        if !system_module_path.is_empty() {
            paths.push(system_module_path.into());
        }
    } else if cfg!(unix) {
        paths.push("/usr/share/numbat/modules".into());
    } else {
        paths.push("C:\\Program Files\\numbat\\modules".into());
    }
    paths
}

fn make_fresh_context() -> Context {
    let mut fs_importer = FileSystemImporter::default();
    for path in get_modules_paths() {
        fs_importer.add_path(path);
    }

    let importer = ChainedImporter::new(
        Box::new(fs_importer),
        Box::<BuiltinModuleImporter>::default(),
    );

    let mut context = Context::new(importer);

    context.set_terminal_width(
        terminal_size::terminal_size().map(|(terminal_size::Width(w), _)| w as usize),
    );

    context
}

#[component]
fn App() -> Element {

    let context = make_fresh_context();
    let context = Arc::new(Mutex::new(context));
    let context_clone = context.clone();
    let mut ctx = context_clone.lock().unwrap();
    let load_result = ctx.interpret("use prelude", CodeSource::Internal);

    let mut input_expression = use_signal(|| "".to_string());
    let mut evaluated_output = use_signal(|| "".to_string());
    let mut interpret_output = use_signal(|| "".to_string());

    rsx! {
        div {
            h1 { "Numbox" }
            input {
                // value: "{input_expression}",
                oninput: move |evt| input_expression.set(evt.value())
                // r#type: "file",
                // accept: ".csv",
                // onchange: move |_evt| {}
            }
            textarea {
                readonly: "true",
                value: "{evaluated_output}"
            }
            textarea {
                readonly: "true",
                value: "{interpret_output}"
            }
            button {
                onclick: move |_| {
                    println!("Evaluating: {}", input_expression.read());
                    let input = input_expression.read();
                    if input.is_empty() {
                        return;
                    }
                    let mut ctx = context.lock().unwrap();
                    let result = ctx.interpret(&input, CodeSource::Text);
                    match result {
                        Ok((statements, interpreter_result)) => {
                            evaluated_output.set(
                                statements.iter()
                                    .map(|s| s.pretty_print())
                                    .collect::<Vec<_>>()
                                    .into_iter()
                                    .join("\n")
                            );
                            interpret_output.set(
                                interpreter_result.to_markup(
                                    statements.last(),
                                    ctx.dimension_registry(),
                                    true,
                                    true
                                ).to_string()
                            );
                        }
                        Err(e) => {
                            evaluated_output.set(format!("{}", e));
                        }
                    }
                },
                "eval"
            }
            // table {tr{td{input{}}}}
        }
    }
}


fn main() {
    dioxus::launch(App);
}
