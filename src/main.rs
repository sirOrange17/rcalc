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

#[derive(PartialEq)]
enum Mode {
    Evaluate,
    Define,
}

#[derive(PartialEq, PartialOrd)]
enum EvalResult {
    Num(f64),
    Comp(bool)
}

fn main() {
	let mut args: Vec<String> = env::args().collect();
	args.remove(0);
	let mut args: String = args.into_iter().collect();
	args.retain(|c| !c.is_whitespace());
	
	if args != "" {
        match parse_txt(&args, &0.0) {
            Ok(result) => {
                match result {EvalResult::Num(result_num) => println!("{}", result_num), EvalResult::Comp(result_bool) => println!("{}", result_bool)}
            }
            Err(e) => println!("Error: {}", e),
        }
		return;
	}
	
    let mut ans: f64 = 0.0;
    let mut mode: Mode = Mode::Evaluate;

    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line.");
        input.retain(|c| !c.is_whitespace());
        
        if input == "exit" {
            break;
        } else if input == "eval" {
            mode = Mode::Evaluate;
            continue;
        } else if input == "defn" {
            mode = Mode::Define;
            continue;
        }

        let cols = termsize::get().unwrap().cols as usize;

        match mode {
            Mode::Evaluate => {
                match parse_txt(&input, &ans) {
                    Ok(result) => {
                        match result {
                            EvalResult::Num(result_num) => {ans = result_num; println!("{:>cols$}", ans);}
                            EvalResult::Comp(result_bool) => {println!("{:>cols$}", result_bool);}
                        }
                    }
                    Err(e) => println!("Error: {}", e),}
            }
            Mode::Define => todo!()
        }
    }
}

fn equation_comparison(txt: &str, comp_index: Option<usize>) -> Result<EvalResult,String> {
    let comparator = txt.as_bytes()[comp_index.unwrap()] as char;
    let mut sides: Vec<&str> = txt.split(comparator).collect();
    sides.retain(|val|val != &"");
    if sides.len() != 2 {
        return Err("Can only compare 2 equations".into());
    }
    let left_side = sides[0];
    let right_side = sides[1];

    let left_ans = 0.0;
    let right_ans = 0.0;
    
    let left_result = parse_txt(left_side, &left_ans)?;
    let right_result = parse_txt(right_side, &right_ans)?;

    if comparator == '=' {
        Ok(EvalResult::Comp(left_result == right_result))
    } else if comparator == '>' {
        Ok(EvalResult::Comp(left_result > right_result))
    } else if comparator == '<' {
        Ok(EvalResult::Comp(left_result < right_result))
    } else {
        Err("Comparison Failed".into())
    }
}

fn parse_txt(txt: &str, ans: &f64) -> Result<EvalResult, String> {
    let comp_index = txt.bytes().position(|val| {(val == b'=')|(val == b'<')|(val == b'>')});
    if comp_index == None {
        let tokens = tokenize(txt, ans)?;
        evaluate_expression(&tokens)
    } else {
        equation_comparison(txt, comp_index)
    }
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

fn evaluate_expression(tokens: &[Token]) -> Result<EvalResult, String> {
    let (result, _) = parse_tokens(tokens, 0)?;
    Ok(EvalResult::Num(result))
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