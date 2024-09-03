#![feature(try_blocks)]
#![feature(exit_status_error)]

use rust_format::{Formatter, RustFmt};
use rvm_core::{MethodDescriptor, ObjectType, PrimitiveType, Type};
use rvm_reader::ClassInfo;
use std::collections::HashMap;
use std::fmt::Write;
use std::fs::{metadata, read, read_dir, File};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::string::String;
use std::{env, fmt, fs, io};
//let item_struct: ItemStruct = parse(item.clone()).unwrap();
//
// 	let package = attr.to_string();
// 	let class_name = item_struct.ident.to_string();
//
// 	let mut bytecode_dir =
// 		PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("bytecode");
// 	for package in package.split("/") {
// 		bytecode_dir.push(package);
// 	}
// 	bytecode_dir.push(format!("{class_name}.class"));
//
// 	if !bytecode_dir.exists() {
// 		panic!(".class file {bytecode_dir:?} does not exist.");
// 	}
//
// 	let file = read(&bytecode_dir).unwrap();
// 	let info = ClassInfo::parse_complete(&file).unwrap();
//
// 	for method in info.methods {}
// 	item

fn get_paths() -> Vec<PathBuf> {
	let mut paths = vec![];
	walk_dir(PathBuf::from("src"), &mut paths);
	paths.retain(|v| {
		let Some(extension) = v.extension() else {
			return false;
		};
		let Some(extension) = extension.to_str() else {
			return false;
		};

		extension == "java"
	});
	paths
}

fn check_needs_recompile(paths: &[PathBuf]) -> bool {
	let mut needs_recompile = false;

	for path in paths {
		let java_file = File::open(path)
			.unwrap()
			.metadata()
			.unwrap()
			.modified()
			.unwrap();
		let class_path = format!(
			"bytecode/{}.class",
			path.to_str()
				.unwrap()
				.trim_start_matches("src/")
				.trim_end_matches(".java")
		);
		let class_path = PathBuf::from(class_path);

		if class_path.exists() {
			match metadata(&class_path) {
				Ok(class_file) => match class_file.modified() {
					Ok(class_modified) => {
						if java_file <= class_modified {
							continue;
						}
					}
					Err(_) => {
						println!("cargo:warning=Could not get modified time");
					}
				},
				_ => {
					println!("cargo:warning=Could not find file at {class_path:?}");
				}
			}
		} else {
			println!("cargo:warning={class_path:?} does not exist");
		}

		println!("cargo:warning={path:?} needs recompiling");
		needs_recompile = true;
	}
	needs_recompile
}

fn compile_java(files: &[PathBuf], current_dir: &Path) {
	let mut process = Command::new(match std::env::var("JAVA_HOME") {
		Ok(java_home) => {
			println!("Using JDK: \"{java_home}\"",);
			format!("{}/bin/javac", java_home)
		}
		_ => "javac".to_string(),
	});

	println!(
		"Using JAVAC: \"{}\"",
		process.get_program().to_str().unwrap()
	);
	process.current_dir(current_dir.join("src")).arg("-Xlint");

	process.arg("-XDignore.symbol.file=true");

	let bytecode_dir = current_dir.join("bytecode");
	std::fs::create_dir_all(&bytecode_dir).expect("Could not create bytecode dir");
	println!("{:?}", bytecode_dir.canonicalize().unwrap());
	process.args(["-d", "../bytecode"]);
	//process.args(["--patch-module", "java.base=java/lang"]);
	//process.args(["--system", "none"]);

	for path in files {
		let path: PathBuf = path.components().collect();
		let canonic_path = path.canonicalize().unwrap();

		process.arg(canonic_path);
	}

	let status = process.status().expect("Could not start java compiler");

	let string = String::from_utf8(process.output().unwrap().stderr).unwrap();
	if !string.trim().is_empty() {
		for string in string.split("\n") {
			println!("cargo:warning=JAVAC: {string}",);
		}
	}
	status.exit_ok().expect("javac not successful");
}

fn main() {
	eprintln!("HI!");
	let paths = get_paths();
	let needs_recompile = check_needs_recompile(&paths);

	//if !needs_recompile {
	//	return;
	//}

	let current_dir = std::env::current_dir().unwrap();
	compile_java(&paths, &current_dir);

	let mut root = PackageNode::default();
	for x in paths {
		let simple: PathBuf = x.components().skip(1).collect();
		let mut buf = PathBuf::from("bytecode").join(simple);
		buf.set_extension("class");

		let vec = read(&buf).unwrap();
		let info = ClassInfo::parse_complete(&vec).unwrap();

		root.insert(ClassData { info, source: buf });
	}

	let mut code_out = String::new();
	let compiler = RustCompiler { root };
	compiler.write_node(&compiler.root, &mut code_out).unwrap();

	let formatted_code = RustFmt::default().format_str(&code_out).unwrap_or(code_out);
	let out_dir = env::var_os("OUT_DIR").unwrap();
	let dest_path = Path::new(&out_dir).join("java_bindings.rs");

	fs::write(&dest_path, formatted_code).unwrap();

	println!("cargo::rerun-if-changed=build.rs");
}

pub struct ClassData {
	info: ClassInfo,
	source: PathBuf,
}
#[derive(Default)]
pub struct PackageNode {
	full_name: String,
	packages: HashMap<String, PackageNode>,
	file: Option<ClassData>,
}

impl PackageNode {
	pub fn insert(&mut self, data: ClassData) {
		let name = data.info.full_name();

		let path_parts: Vec<String> = name.split("/").map(|v| v.to_string()).collect();

		self.insert_raw(path_parts, data);
	}

	fn insert_raw(&mut self, mut parts: Vec<String>, data: ClassData) {
		if parts.len() == 0 {
			if let Some(existing) = &self.file {
				panic!(
					"Conflicting class name {} {:?} with {:?}",
					data.info.full_name(),
					existing.source,
					data.source,
				);
			}

			self.file = Some(data);
		} else {
			let package = parts.remove(0);
			let package_name = format!("{}/{package}", self.full_name)
				.trim_start_matches("/")
				.to_string();
			let node = self.packages.entry(package).or_default();
			node.full_name = package_name;
			node.insert_raw(parts, data);
		}
	}
}

pub struct RustCompiler {
	root: PackageNode,
}

impl RustCompiler {
	pub fn class_to_rust(&self, package: &str, class: &str) -> String {
		class
			.trim_start_matches(package)
			.replace("/", "::")
			.trim_start_matches("::")
			.to_string()
	}
	pub fn write_type(&self, ty: &Type, package: &str, out: &mut String) {
		match ty {
			Type::Primitive(primitive) => match primitive {
				PrimitiveType::Boolean => out.push_str("bool"),
				PrimitiveType::Byte => out.push_str("i8"),
				PrimitiveType::Short => out.push_str("i16"),
				PrimitiveType::Int => out.push_str("i32"),
				PrimitiveType::Long => out.push_str("i64"),
				PrimitiveType::Char => out.push_str("char"),
				PrimitiveType::Float => out.push_str("f32"),
				PrimitiveType::Double => out.push_str("f64"),
			},
			Type::Object(object) => {
				if object == &ObjectType::Object() {
					out.push_str("rvm_runtime::Reference");
				} else {
					out.push_str("rvm_runtime::Instance<");
					out.push_str(&self.class_to_rust(package, object));
					out.push('>');
				}
			}
			Type::Array(array) => {
				out.push_str("rvm_runtime::Array<");
				self.write_type(&array.component, package, out);
				out.push('>');
			}
		}
	}
	pub fn write_node(&self, node: &PackageNode, out: &mut String) -> fmt::Result {
		let parent_node = node;
		for (key, node) in &node.packages {
			if let Some(data) = &node.file {
				let info = &data.info;
				let full_name = info.full_name();
				let (package, name) = match full_name.rsplit_once("/") {
					Some((package, name)) => (package.to_string(), name.to_string()),
					None => (String::new(), full_name.clone()),
				};

				let super_class =
					info.constant_pool[info.constant_pool[info.super_class].name].to_string();
				let super_name = if super_class != *ObjectType::Object() {
					Some(self.class_to_rust(&package, &super_class))
				} else {
					None
				};

				writeln!(out, "// {}", parent_node.full_name)?;
				writeln!(out, "#[derive(Copy, Clone)]")?;
				writeln!(out, "pub struct {name} {{")?;
				if let Some(super_name) = &super_name {
					writeln!(out, "  base: {super_name},")?;
				}
				for field in &info.fields {
					let field_name = info.constant_pool[field.name_index].to_string();
					let field_descriptor = info.constant_pool[field.descriptor_index].to_string();
					let field_type = Type::parse(&field_descriptor).unwrap();
					write!(out, "  pub {field_name}: rvm_runtime::TypedField<")?;
					self.write_type(&field_type, &package, out);
					writeln!(out, ">,")?;
				}
				writeln!(out, "}}")?;

				// InstanceBinding
				writeln!(out, "impl rvm_runtime::InstanceBinding for {name} {{")?;
				writeln!(out, "  fn ty() -> rvm_core::ObjectType {{")?;
				writeln!(out, "    rvm_core::ObjectType::new(Self::TY)")?;
				writeln!(out, "  }}")?;
				writeln!(
					out,
					"  fn bind(instance: &rvm_runtime::AnyInstance) -> Self {{"
				)?;
				writeln!(out, "    {name} {{")?;
				if let Some(super_name) = &super_name {
					writeln!(out, "    base: {super_name}::bind(instance),")?;
				}
				for field in &info.fields {
					let field_name = info.constant_pool[field.name_index].to_string();
					writeln!(out, "    {field_name}: instance.field_named(\"{field_name}\").unwrap().typed(),")?;
				}
				writeln!(out, "    }}")?;
				writeln!(out, "  }}")?;
				writeln!(out, "}}")?;

				if let Some(super_name) = &super_name {
					writeln!(
						out,
						"impl std::ops::Deref for {name} {{
						type Target = {super_name};

						fn deref(&self) -> &Self::Target {{
							&self.base
						}}
					}}

					impl std::ops::DerefMut for {name} {{
						fn deref_mut(&mut self) -> &mut Self::Target {{
							&mut self.base
						}}
					}}"
					)?;
				}

				//#[derive(Clone, Copy)]
				// pub struct ExtendedObject {
				// 	base: SimpleObject,
				// 	another_field: TypedField<i64>,
				// }
				//
				// impl InstanceBinding for ExtendedObject {
				// 	fn ty() -> ObjectType {
				// 		ObjectType::new("testing/object/ExtendedObject")
				// 	}
				//
				// 	fn bind(instance: &AnyInstance) -> Self {
				// 		ExtendedObject {
				// 			base: SimpleObject::bind(instance),
				// 			another_field: instance.field_named("anotherField").unwrap().typed(),
				// 		}
				// 	}
				// }
				// Methods
				writeln!(out, "impl {name} {{")?;
				writeln!(out, "	pub const TY: &'static str =\"{full_name}\";")?;

				// Check what method names need descriptors
				let mut method_names = HashMap::<String, usize>::new();
				for method in &info.methods {
					let method_name = info.constant_pool[method.name_index].to_string();
					*method_names.entry(method_name).or_default() += 1;
				}
				for method in &info.methods {
					let method_name = info.constant_pool[method.name_index].to_string();
					let descriptor = info.constant_pool[method.descriptor_index].to_string();
					let descriptor = MethodDescriptor::parse(&descriptor).unwrap();

					let mut rust_method_name = method_name.replace(['<', '>'], "_");
					if *method_names.get(&method_name).unwrap() > 1 {
						let parameters: Vec<String> = descriptor
							.parameters
							.iter()
							.map(|v| v.to_string())
							.collect();
						for char in parameters.join("").to_string().chars() {
							if char.is_ascii_alphanumeric() {
								rust_method_name.push(char);
							} else {
								rust_method_name.push('_');
							}
						}
					}

					// Descriptor method
					writeln!(
						out,
						"pub fn {rust_method_name}_descriptor() -> rvm_runtime::MethodIdentifier {{"
					)?;

					writeln!(out, "rvm_runtime::MethodIdentifier {{")?;
					writeln!(out, "	name: std::sync::Arc::from(\"{method_name}\"),")?;
					writeln!(out, "	descriptor: std::sync::Arc::from(\"{descriptor}\"),")?;
					writeln!(out, "}}")?;
					writeln!(out, "}}")?;

					// Simple call method

					write!(
						out,
						"pub fn {rust_method_name}(runtime: &rvm_runtime::Runtime,"
					)?;

					for (i, ty) in descriptor.parameters.iter().enumerate() {
						write!(out, "v{i}: ")?;
						self.write_type(ty, &parent_node.full_name, out);
						write!(out, ",")?;
					}
					write!(out, ") -> eyre::Result<")?;
					match &descriptor.returns {
						None => {
							out.push_str("()");
						}
						Some(value) => {
							self.write_type(value, &parent_node.full_name, out);
						}
					}
					writeln!(out, "> {{")?;

					// BODY {
					writeln!(out, "let output = runtime.simple_run(<Self as rvm_runtime::InstanceBinding>::ty(), Self::{rust_method_name}_descriptor(),")?;

					writeln!(out, "vec![")?;
					for (i, _ty) in descriptor.parameters.iter().enumerate() {
						writeln!(out, "rvm_runtime::ToJava::to_java(v{i}, runtime)?,")?;
					}
					writeln!(out, "],)?;")?;

					match descriptor.returns {
						Some(_) => {
							writeln!(out, "let output = output.expect(\"expected return\");")?;
							writeln!(out, "rvm_runtime::FromJava::from_java(output, runtime)")?;
						}
						None => {
							writeln!(
								out,
								"if !output.is_none() {{ panic!(\"Returned on void\"); }}"
							)?;
							writeln!(out, "	Ok(())")?;
						}
					}
					writeln!(out, "}}")?;
					//}
				}
				// Java ObjectType
				//writeln!(out, "pub fn ty() -> rvm_core::ObjectType {{")?;
				//writeln!(out, "  rvm_core::ObjectType::new(Self::TY)")?;
				//writeln!(out, "}}")?;
				writeln!(out, "}}")?;

				// Java Typed
				writeln!(out, "impl rvm_runtime::JavaTyped for {name} {{")?;
				writeln!(out, "  fn java_type() -> rvm_core::Type {{")?;
				writeln!(
					out,
					"    <Self as rvm_runtime::InstanceBinding>::ty().into()"
				)?;
				writeln!(out, "  }}")?;
				writeln!(out, "}}")?;
			} else {
				writeln!(out, "pub mod {key} {{")?;
				self.write_node(node, out)?;
				writeln!(out, "}}")?;
			}
		}

		Ok(())
	}
}

fn walk_dir(path: PathBuf, paths: &mut Vec<PathBuf>) {
	for dir in read_dir(&path).unwrap() {
		let entry = dir.unwrap();
		if entry.path().extension().map(|v| v.to_str().unwrap()) == Some("java") {
			paths.push(entry.path());
		} else if entry.metadata().unwrap().is_dir() {
			walk_dir(path.join(entry.file_name()), paths);
		}
	}
}
