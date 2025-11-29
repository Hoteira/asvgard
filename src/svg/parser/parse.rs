use crate::svg::parser::tags::Tag;

pub fn load_xml(data: &[u8]) -> Vec<Tag> {
    let (_, total_tags, _) = load_xml_recursive(data, 0);
    total_tags
}

pub fn load_xml_recursive(data: &[u8], idx: usize) -> (usize, Vec<Tag>, String) {
    let mut name = Vec::new();
    let mut name_start = false;
    let mut params_start = false;
    let mut text_buffer = String::new();
    let mut parent_content = String::new();

    let mut total_tags: Vec<Tag> = Vec::new();
    let mut tag = Tag::new();

    let mut i = idx;
    while i < data.len() {
        if data[i] == '<' as u8 {
            if !text_buffer.is_empty() {
                 parent_content.push_str(&text_buffer);
                 text_buffer.clear();
            }

            if data[i + 1] == '/' as u8 {
                if tag.name != "" {
                    total_tags.push(tag.clone());
                    tag.clear();
                }

                while i < data.len() && data[i] != '>' as u8 {
                    i += 1;
                }
                
                return (i + 1, total_tags, parent_content.trim().to_string());

            } else if data[i + 1] == '!' as u8 {
                while i < data.len() && data[i] != '>' as u8 {
                    i += 1;
                }
            } else {
                name_start = true;
                params_start = false;
            }

        } else if name_start {
            if data[i] == ' ' as u8 || data[i] == '>' as u8 {
                name_start = false;
                tag.name = sanitize(String::from_utf8(name.clone()).unwrap());
                name.clear();

                if data[i] == '>' as u8 {
                    params_start = false;
                    i -= 1;
                } else {
                    params_start = true;
                }
            } else {
                name.push(data[i]);
            }

        } else if data[i] == '>' as u8 {
            if data[i - 1] == ']' as u8 {
                i += 1;
                continue;
            }

            name_start = false;
            params_start = false;

            let params = String::from_utf8(name.clone()).unwrap();
            if !params.is_empty() {
                let mut args = Vec::new();
                let mut values = Vec::new();
                handle_params(&params, &mut args, &mut values);

                for j in 0..args.len() {
                    tag.params.insert(
                        sanitize(args[j].clone()),
                        values[j].clone()
                    );
                }
            }

            name.clear();

            if data[i - 1] == '/' as u8 {
                total_tags.push(tag.clone());
                tag.clear();
            } else {
                if !tag.name.is_empty() {
                    let (next_i, children, content) = load_xml_recursive(data, i + 1);
                    tag.children = children;
                    tag.text_content = content;
                    total_tags.push(tag.clone());
                    tag.clear();

                    i = next_i;
                    continue;
                }
            }

        } else if params_start {
            name.push(data[i]);
        } else {
            text_buffer.push(data[i] as char);
        }

        i += 1;
    }

    (i, total_tags, parent_content.trim().to_string())
}

fn handle_params(p0: &String, args: &mut Vec<String>, values: &mut Vec<String>) {
    let mut arg = String::new();
    let mut value = String::new();

    let p0 = p0.as_bytes();

    let mut getting_value = false;
    let mut getting_arg = true;

    let mut value_start = '*';
    for i in 0..p0.len() {

        if getting_arg {
            value_start = '*';
            if p0[i] != '=' as u8 {
                arg.push(p0[i] as char);
            } else {
                args.push(arg.clone());
                arg.clear();
                getting_arg = false;
                getting_value = true;
            }
        } else if getting_value {
            if value_start == '*' && value.len() == 0 {
                if p0[i] == '"' as u8 || p0[i] == '\'' as u8 {
                    value_start = p0[i] as char;
                }

            } else {
                if p0[i] == value_start as u8 {
                    values.push(value.clone());
                    value.clear();
                    getting_arg = true;
                    getting_value = false;
                } else {
                    value.push(p0[i] as char);
                }
            }
        }
    }
}

pub fn sanitize(s: String) -> String {
    s.replace(' ', "")
        .replace('\n', "")
        .replace('\t', "")
        .replace('\r', "")
        .replace('/', "")
        .replace('<', "")
        .replace('>', "")
}
