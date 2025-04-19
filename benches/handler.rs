use criterion::{Criterion, black_box, criterion_group, criterion_main};
use marky::handler::Handle;

fn benchmark_handler_new(c: &mut Criterion) {
    let code = [
        "# Title",
        "",
        "Hello, world!",
        "",
        "## Subtitle",
        "",
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
        "### Subtitle 2",
        "",
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
        "### Subtitle 3",
        "",
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do *eiusmod* tempor incididunt ut labore et dolore magna aliqua.",
        "",
        "> This is a blockquote",
        "> ",
        "> Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
        "",
    ];

    let code = code.join("\n");

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(tree_sitter_md::language())
        .expect("Failed to set language");

    c.bench_function("handler.new", |b| {
        b.iter(|| {
            let handle = Handle::new(black_box(&code), black_box(&mut parser)).expect("Failed to create handle");
            let _ = black_box(handle);
        });
    });
}


fn benchmark_handler_update_no_change(c: &mut Criterion) {
    let code = [
        "# Title",
        "",
        "Hello, world!",
        "",
        "## Subtitle",
        "",
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
        "### Subtitle 2",
        "",
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
        "### Subtitle 3",
        "",
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do *eiusmod* tempor incididunt ut labore et dolore magna aliqua.",
        "",
        "> This is a blockquote",
        "> ",
        "> Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
        "",
    ];

    let code = code.join("\n");

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(tree_sitter_md::language())
        .expect("Failed to set language");

    c.bench_function("handler.update.no_change", |b| {
        let mut handler = Handle::new(&code, &mut parser).expect("Failed to create handle");

        b.iter(|| {
            let handle = black_box(&mut handler);
            let code = black_box(&code);
            let parser = black_box(&mut parser);

            handle.update(code, parser).expect("Failed to update handle");
        })
    });
}


criterion_group!(benches, benchmark_handler_new, benchmark_handler_update_no_change);
criterion_main!(benches);
