use Realm::compile_source;

fn main() {
    println!("Hello, world!");
    let source_code: &str = "let x = 42 + 10";
    println!("Compiling: {}", source_code);
    compile_source(source_code);
}
