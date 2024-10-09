use std::io;
use std::env;

#[derive(Debug)]
enum Token {
    Number(f64),
    Operator(char),
    LeftParen,
    RightParen,
    Constant(String),
    Function(String),
}

fn main() {
	let mut args: Vec<String> = env::args().collect();
	args.remove(0);
	let mut args: String = args.into_iter().collect();
	args.retain(|c| !c.is_whitespace());
	
	if args != "" {
        match parse_txt(&args, &0.0) {
            Ok(result) => {
                println!("{}", result);
            }
            Err(e) => println!("Error: {}", e),
        }
		return;
	}
	
    let mut ans: f64 = 0.0;
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line.");
        input.retain(|c| !c.is_whitespace());

        if input == "exit" {
            break;
        }

        let cols = termsize::get().unwrap().cols as usize;
        match parse_txt(&input, &ans) {
            Ok(result) => {
                ans = result;
                println!("{:>cols$}", ans);
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}

fn parse_txt(txt: &str, ans: &f64) -> Result<f64, String> {
    let tokens = tokenize(txt, ans)?;
    evaluate_expression(&tokens)
}

fn tokenize(txt: &str, ans: &f64) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = txt.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            '0'..='9' | '.' => {
                let mut num_str = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_digit(10) || ch == '.' {
                        num_str.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(num_str.parse().map_err(|_| "Invalid number format")?));
            }
            'a' if chars.clone().collect::<String>().starts_with("ans") => {
                tokens.push(Token::Number(*ans));
                chars.next();
                chars.next();
				chars.next();
			}
            '+' | '*' | '/' | '^' => {
                tokens.push(Token::Operator(ch));
                chars.next();
            }
            '-' => {
                chars.next();
                if tokens.is_empty() || matches!(tokens.last(), Some(Token::Operator(_)) | Some(Token::LeftParen)) {
                    if let Some(&next_ch) = chars.peek() {
                        if next_ch.is_digit(10) || next_ch == '.' {
                            let mut num_str = String::new();
                            num_str.push('-');
                            while let Some(&ch) = chars.peek() {
                                if ch.is_digit(10) || ch == '.' {
                                    num_str.push(ch);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            tokens.push(Token::Number(num_str.parse().map_err(|_| "Invalid number format")?));
                        } else {
                            return Err("Unexpected character after '-'".into());
                        }
                    } else {
                        return Err("Unexpected end of input after '-'".into());
                    }
                } else {
                    tokens.push(Token::Operator('-'));
                }
            }
            '(' => {
                tokens.push(Token::LeftParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RightParen);
                chars.next();
            }
            'p' if chars.clone().collect::<String>().starts_with("pi") => {
                tokens.push(Token::Constant("pi".to_string()));
                chars.next();
                chars.next();
            }
            'e' if !chars.clone().collect::<String>().starts_with("exp") => {
                tokens.push(Token::Constant("e".to_string()));
                chars.next();
            }
            _ if ch.is_alphabetic() => {
                let mut func_name = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphabetic() {
                        func_name.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Function(func_name));
            }
            _ => {
                return Err(format!("Unexpected character: {}", ch));
            }
        }
    }
    Ok(tokens)
}

fn evaluate_expression(tokens: &[Token]) -> Result<f64, String> {
    let (result, _) = parse_tokens(tokens, 0)?;
    Ok(result)
}

fn parse_tokens(tokens: &[Token], mut index: usize) -> Result<(f64, usize), String> {
    let mut values = Vec::new();
    let mut ops = Vec::new();

    while index < tokens.len() {
        match &tokens[index] {
            Token::Number(num) => {
                values.push(*num);
                index += 1;
            }
            Token::Constant(name) => {
                let value = match name.as_str() {
                    "pi" => std::f64::consts::PI,
                    "e" => std::f64::consts::E,
                    _ => return Err(format!("Unknown constant: {}", name)),
                };
                values.push(value);
                index += 1;
            }
            Token::Function(name) => {
                let (arg, new_index) = parse_tokens(tokens, index + 1)?;
                let value = match name.as_str() {
                    "sin" => arg.sin(),
                    "cos" => arg.cos(),
					"tan" => arg.tan(),
                    "asin" => arg.asin(),
                    "acos" => arg.acos(),
					"atan" => arg.atan(),
                    "sqrt" => arg.sqrt(),
					"abs" => arg.abs(),
					"ln" => arg.ln(),
					"exp" => arg.exp(),
                    _ => return Err(format!("Unknown function: {}", name)),
                };
                values.push(value);
                index = new_index;
            }
            Token::Operator(op) => {
                while !ops.is_empty() && precedence(ops.last().unwrap()) >= precedence(op) {
                    apply_operator(&mut values, ops.pop().unwrap())?;
                }
                ops.push(*op);
                index += 1;
            }
            Token::LeftParen => {
                let (val, new_index) = parse_tokens(tokens, index + 1)?;
                values.push(val);
                index = new_index;
            }
            Token::RightParen => {
                while !ops.is_empty() {
                    apply_operator(&mut values, ops.pop().unwrap())?;
                }
                return Ok((values.pop().ok_or("Mismatched parentheses")?, index + 1));
            }
        }
    }

    while !ops.is_empty() {
        apply_operator(&mut values, ops.pop().unwrap())?;
    }

    Ok((values.pop().ok_or("Invalid expression")?, index))
}

fn precedence(op: &char) -> i32 {
    match op {
        '+' | '-' => 1,
        '*' | '/' => 2,
        '^' => 3,
        _ => 0,
    }
}

fn apply_operator(values: &mut Vec<f64>, op: char) -> Result<(), String> {
    let b = values.pop().ok_or("Insufficient values in expression")?;
    let a = values.pop().ok_or("Insufficient values in expression")?;
    let result = match op {
        '+' => a + b,
        '-' => a - b,
        '*' => a * b,
        '/' => a / b,
        '^' => a.powf(b),
        _ => return Err(format!("Unexpected operator: {}", op)),
    };
    values.push(result);
    Ok(())
}