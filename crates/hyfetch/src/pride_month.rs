fn  strip_trailing_nl(input: &mut String) {
    let strip_len = input
        .char_indices()
        .rev()
        .find(|(_, c)| !matches!(c, '\n' | '\r'))
        .map_or(0, |(i, _)| i + 1);
    if strip_len != input.len() {
        input.truncate(strip_len);
    }
}

fn  start_animation() {
    let mut text =  r#"
.======================================================.
| .  .              .__       .     .  .       , .   | |
| |__| _.._ ._   .  [__)._.* _| _   |\/| _ ._ -+-|_  | |
| |  |(_][_)[_)\_|  |   [  |(_](/,  |  |(_)[ ) | [ ) * |
|        |  |  ._|                                     |
'======================================================'"#.to_string();

    strip_trailing_nl(&mut text);
    let text_lines = text.split("\n");
    //let text_height = text_lines.len();
    //let text_width = text_lines[0].len();
    println!

    for elem in text_lines {
        println!("elem: {}", elem);
    }
}

fn  main() {
    start_animation();
}
