mod tags;

fn main() {

    let svg_data = include_bytes!("../icon.svg");

    let mut name = Vec::new();
    let mut tag_start = false;
    let mut tags_open = 0;

    for data in svg_data {
        if *data == '<' as u8 {
            tag_start = true;
            tags_open += 1;
        } else if tag_start {
            if *data == ' ' as u8 || *data == '>' as u8 {
                tag_start = false;

                println!("{}", String::from_utf8(name.clone()).unwrap());
                name.clear();

                if *data == '>' as u8 { tags_open -= 1 };

            } else if *data == '/' as u8 {
                tag_start = false;
            } else {
                name.push(*data);
            }
        }
    }


    println!("Hello, world!");
}
