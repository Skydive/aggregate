use {
	std::{sync::Arc, collections::HashMap},
	swc::{
		Compiler
	},
	swc_common::{
		FilePathMapping, SourceMap, FileName, Globals,
		errors::{ColorConfig, Handler}
	},
	swc_bundler::{
		BundleKind,	Bundler, Config
	},
	swc_ecma_codegen::{
		Emitter,
		text_writer::JsWriter
	},

	spack::{
		resolvers::NodeResolver,
		loaders::swc::SwcLoader
	}
};



pub fn compile_file(file_path: String, bmap: HashMap<String, std::path::PathBuf>, wr: Box<dyn std::io::Write + 'static>) -> Result<Vec<swc_bundler::Bundle>, Box<dyn std::error::Error + 'static>> {
    let globals = Globals::new();
    let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));
    let handler = Arc::new(Handler::with_tty_emitter(
        ColorConfig::Always,
        true,
        false,
        Some(cm.clone()),
    ));

    let c = Arc::new(Compiler::new(cm.clone(), handler));

    // This example does not use core modules.
    let external_modules = vec![];
    let bundler = Bundler::new(
        &globals,
        cm.clone(),
        SwcLoader::new(c, Default::default()),
        NodeResolver::new(),
        Config {
            require: true,
            external_modules,
            ..Default::default()
        },
    );
    let mut entries = HashMap::default();
    for (k, v) in bmap.iter() {
        //entries.insert("main".to_string(), FileName::Real(file_path.into()));
        entries.insert(k.to_string(), FileName::Real(v.to_path_buf()));
    }

    Ok(bundler.bundle(entries).expect("failed to bundle"))
    // assert_eq!(
    //     bundles.len(),
    //     1,
    //     "There's no conditional / dynamic imports and we provided only one entry"
    // );
    // let bundle = bundles.pop().unwrap();
    // assert_eq!(
    //     bundle.kind,
    //     BundleKind::Named {
    //         name: "main".into()
    //     },
    //     "We provided it"
    // );

    //let wr = stdout();
 //    let mut emitter = Emitter {
 //        cfg: swc_ecma_codegen::Config { minify: false },
 //        cm: cm.clone(),
 //        comments: None,
 //        wr: Box::new(JsWriter::new(cm.clone(), "\n", wr, None)),
 //    };

 //    emitter.emit_module(&bundle.module)?;

	// Ok(())
}