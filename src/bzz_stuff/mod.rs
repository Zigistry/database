use std::iter::Peekable;
use std::vec::IntoIter;
use crate::custom_types::Dependency;

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Dot,
    LBrace,
    RBrace,
    Equals,
    Comma,
    Str(String),
    Identifier(String),
    Binary(String),
    Octal(String),
    Hexadecimal(String),
    Integer(String),
}

#[derive(Debug, PartialEq)]
pub struct BuildZigZonData {
    pub name: String,
    pub fingerprint: String,
    pub version: String,
    pub minimum_zig_version: String,
    pub dependencies: Vec<Dependency>,
    pub paths: Vec<String>,
}

pub fn tokenize(
    build_zig_zon_raw_data: &mut Peekable<IntoIter<char>>,
) -> Result<Vec<TokenType>, String> {
    let mut tokens: Vec<TokenType> = Vec::new();

    while let Some(c) = build_zig_zon_raw_data.next() {
        match c {
            c if c.is_whitespace() => {
                continue;
            }
            '/' => {
                if build_zig_zon_raw_data.next_if_eq(&'/').is_some() {
                    // this is a comment
                    while let Some(c) = build_zig_zon_raw_data.next() {
                        if c == '\n' {
                            break;
                        }
                    }
                } else {
                    return Err(
                        "Found a '/' i.e division symbol not supported in build.zig.zon."
                            .to_string(),
                    );
                }
            }
            '.' => tokens.push(TokenType::Dot),
            '{' => tokens.push(TokenType::LBrace),
            '}' => tokens.push(TokenType::RBrace),
            '=' => tokens.push(TokenType::Equals),
            ',' => tokens.push(TokenType::Comma),
            '"' => {
                let mut any_string = String::new();
                while let Some(c) = build_zig_zon_raw_data.next() {
                    match c {
                        '\\' => {
                            any_string.push(c);
                            if let Some(escaped) = build_zig_zon_raw_data.next() {
                                any_string.push(escaped);
                            }
                        }
                        '"' => break,
                        _ => any_string.push(c),
                    }
                }
                tokens.push(TokenType::Str(any_string));
            }
            c if c.is_alphabetic() || c == '_' => {
                let mut identifier = String::new();
                identifier.push(c);

                while let Some(&next) = build_zig_zon_raw_data.peek() {
                    if next.is_alphanumeric() || next == '_' {
                        identifier.push(build_zig_zon_raw_data.next().unwrap());
                    } else {
                        break;
                    }
                }
                tokens.push(TokenType::Identifier(identifier));
            }
            '0' => {
                let mut my_integer = String::from("0");
                if let Some(c) = build_zig_zon_raw_data.next() {
                    my_integer.push(c);
                    match c {
                        'b' => {
                            while let Some(c) = build_zig_zon_raw_data.peek() {
                                match c {
                                    '0' | '1' | '_' => {
                                        my_integer.push(build_zig_zon_raw_data.next().unwrap())
                                    }
                                    _ => break,
                                }
                            }
                            tokens.push(TokenType::Binary(my_integer));
                        }
                        'o' => {
                            while let Some(c) = build_zig_zon_raw_data.peek() {
                                match c {
                                    '0'..='7' | '_' => {
                                        my_integer.push(build_zig_zon_raw_data.next().unwrap())
                                    }
                                    _ => break,
                                }
                            }
                            tokens.push(TokenType::Octal(my_integer));
                        }
                        'x' => {
                            while let Some(c) = build_zig_zon_raw_data.peek() {
                                match c {
                                    '0'..='9' | 'a'..='f' | 'A'..='F' | '_' => {
                                        my_integer.push(build_zig_zon_raw_data.next().unwrap())
                                    }
                                    _ => break,
                                }
                            }
                            tokens.push(TokenType::Hexadecimal(my_integer));
                        }
                        _ => return Err("Got unknown integer type that starts with 0.".to_string()),
                    }
                } else {
                    return Err("File unexpectedly ended after 0.".to_string());
                }
            }
            '1'..='9' => {
                let mut my_integer = String::from(c);
                while let Some(c) = build_zig_zon_raw_data.peek() {
                    match c {
                        '0'..='9' | '_' => my_integer.push(build_zig_zon_raw_data.next().unwrap()),
                        _ => break,
                    }
                }
                tokens.push(TokenType::Integer(my_integer));
            }
            _ => {}
        }
    }
    Ok(tokens)
}

pub fn parse(tokens: &mut Peekable<IntoIter<TokenType>>) -> Result<BuildZigZonData, String> {
    let mut build_zig_zon_parsed = BuildZigZonData {
        name: String::new(),
        fingerprint: String::new(),
        version: String::new(),
        minimum_zig_version: String::new(),
        dependencies: Vec::new(),
        paths: Vec::new(),
    };
    let mut depth = 0;

    while let Some(c) = tokens.next() {
        match c {
            TokenType::LBrace => {
                depth += 1;
            }
            TokenType::RBrace => {
                depth -= 1;
            }
            TokenType::Identifier(val) => {
                match val.as_str() {
                    "minimum_zig_version" => {
                        if tokens.next_if_eq(&TokenType::Equals).is_none() {
                            return Err(
                                "Expected = after minimum zig version declaration.".to_string()
                            );
                        }
                        if let Some(TokenType::Str(s)) = tokens.next() {
                            build_zig_zon_parsed.minimum_zig_version = s;
                        } else {
                            return Err("Expected string after minimum_zig_version =".to_string());
                        }
                    }
                    "name" => {
                        if tokens.next_if_eq(&TokenType::Equals).is_none() {
                            return Err("Expected = after name declaration.".to_string());
                        }
                        // The issue here is that, in Zig 0.12.0, we have
                        // strings as name, now, in Zig 0.15.0, we have
                        // enum literals as name.
                        if let Some(c) = tokens.peek() {
                            if c == &TokenType::Dot {
                                tokens.next();
                            }
                        }
                        match tokens.next() {
                            Some(TokenType::Str(s)) | Some(TokenType::Identifier(s)) => {
                                build_zig_zon_parsed.name = s;
                            }
                            _ => {
                                return Err(
                                    "Expected string or identifier after name =".to_string()
                                );
                            }
                        }
                    }
                    "version" => {
                        if tokens.next_if_eq(&TokenType::Equals).is_none() {
                            return Err("Expected = after version declaration.".to_string());
                        }
                        if let Some(TokenType::Str(s)) = tokens.next() {
                            build_zig_zon_parsed.version = s;
                        } else {
                            return Err("Expected string after version =".to_string());
                        }
                    }
                    "fingerprint" => {
                        if tokens.next_if_eq(&TokenType::Equals).is_none() {
                            return Err("Expected = after fingerprint declaration.".to_string());
                        }
                        if let Some(TokenType::Hexadecimal(s)) = tokens.next() {
                            build_zig_zon_parsed.fingerprint = s;
                        } else {
                            return Err("Expected hexadecimal after fingerprint =".to_string());
                        }
                    }
                    "dependencies" => {
                        if tokens.next_if_eq(&TokenType::Equals).is_none() {
                            return Err("Expected = after dependencies declaration.".to_string());
                        }
                        if tokens.next_if_eq(&TokenType::Dot).is_none() {
                            return Err("Expected . after dependencies =".to_string());
                        }
                        if tokens.next_if_eq(&TokenType::LBrace).is_none() {
                            return Err("Expected { after dependencies = .".to_string());
                        }
                        depth += 1;

                        let mut dependencies = Vec::new();

                        loop {
                            // Ignoring commas in betweens dependencies
                            tokens.next_if_eq(&TokenType::Comma);

                            // Basically, at the end of the block.
                            if tokens.next_if_eq(&TokenType::RBrace).is_some() {
                                depth -= 1;
                                break;
                            }

                            if tokens.next_if_eq(&TokenType::Dot).is_none() {
                                return Err("Expected . before dependency name".to_string());
                            }

                            let dep_name = match tokens.next() {
                                Some(TokenType::Str(s)) | Some(TokenType::Identifier(s)) => s,
                                _ => return Err("Expected dependency name".to_string()),
                            };

                            if tokens.next_if_eq(&TokenType::Equals).is_none() {
                                return Err("Expected = after dependency name".to_string());
                            }

                            if tokens.next_if_eq(&TokenType::Dot).is_none() {
                                return Err("Expected . after dependency name =".to_string());
                            }

                            if tokens.next_if_eq(&TokenType::LBrace).is_none() {
                                return Err("Expected { for dependency object".to_string());
                            }
                            depth += 1;

                            let mut dependency = Dependency {
                                name: dep_name,
                                url: String::new(),
                                hash: String::new(),
                                lazy: String::new(),
                                path: String::new(),
                            };

                            // parsing the thing inside the dependency = .{ .something = .{...}}
                            loop {
                                tokens.next_if_eq(&TokenType::Comma);

                                // Again, if at the end of the file.
                                if tokens.next_if_eq(&TokenType::RBrace).is_some() {
                                    depth -= 1;
                                    break;
                                }

                                if tokens.next_if_eq(&TokenType::Dot).is_none() {
                                    return Err("Expected . before field name".to_string());
                                }

                                let key = match tokens.next() {
                                    Some(TokenType::Str(s)) | Some(TokenType::Identifier(s)) => s,
                                    _ => return Err("Expected field name".to_string()),
                                };

                                if tokens.next_if_eq(&TokenType::Equals).is_none() {
                                    return Err("Expected = after field name".to_string());
                                }

                                let value = match tokens.next() {
                                    Some(TokenType::Str(s)) | Some(TokenType::Identifier(s)) => s,
                                    _ => return Err("Expected value after field =".to_string()),
                                };

                                match key.as_str() {
                                    "url" => dependency.url = value,
                                    "hash" => dependency.hash = value,
                                    "path" => dependency.path = value,
                                    "lazy" => dependency.lazy = value,
                                    _ => {}
                                }
                            }

                            dependencies.push(dependency);
                        }

                        // Oh no, forgot the main thing XD
                        build_zig_zon_parsed.dependencies = dependencies;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    Ok(build_zig_zon_parsed)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn important() {
        let test = r#"
        // test comment
        // another one
        .{
            .name = "bzz parser",
            .version = "0.0.0",
            .minimum_zig_version = "0.15.1",
            .fingerprint = 0x0123456789abcdef
            .dependencies = .{
                .something = .{
                    .url = "some url",
                    .hash = "some hash",
                    .path = "some path",
                    .lazy = true,
                },
                .something2 = .{
                    .url = "some url",
                    .hash = "some hash",
                    .path = "some path",
                    .lazy = true
                }
            }
        }
        "#;
        let res = tokenize(&mut test.chars().collect::<Vec<_>>().into_iter().peekable()).unwrap();

        let res2 = parse(&mut res.into_iter().peekable());
        println!("{:#?}", res2);
    }
}
