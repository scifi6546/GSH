#[derive(std::cmp::PartialEq, Debug)]
pub enum Token {
    FreeWrite,
    AppendWrite,
    DrawText {
        x: f32,
        y: f32,
        text: String,
    },
    DrawLine {
        x_start: f32,
        y_start: f32,
        x_end: f32,
        y_end: f32,
    },
    Error(SyntaxError),
}
#[derive(std::cmp::PartialEq, Debug)]
pub enum SyntaxError {
    InvalidToken,
}
pub fn parse(data: String) -> Result<Vec<Token>, SyntaxError> {
    let mut tokens = vec![];
    let mut iter = data.split("\n").peekable();
    loop {
        let line = iter.peek();
        if line.is_none() {
            return Ok(tokens);
        }
        let push_token = match line.unwrap() {
            &"SET FREE" => {
                iter.next();
                Some(Token::FreeWrite)
            }
            &"SET APPEND" => {
                iter.next();
                Some(Token::AppendWrite)
            }
            &"" => {
                iter.next();
                None
            }
            _ => {
                iter.next();
                Some(Token::Error(SyntaxError::InvalidToken))
            }
        };
        if let Some(token) = push_token {
            tokens.push(token);
        }
    }
    //should never rach this point
    panic!();
    return Err(SyntaxError::InvalidToken);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty() {
        assert_eq!(parse(String::new()), Ok(vec![]));
    }
    #[test]
    fn set_free() {
        let p = parse("SET FREE\nSET APPEND".to_string());
        assert_eq!(p, Ok(vec![Token::FreeWrite, Token::AppendWrite]));
    }
    #[test]
    fn text() {
        let p = parse("\"\nhello world\n\"\nDRAW TEXT(10,10)\n\"hello world\n\"".to_string());
        assert_eq!(
            p,
            Ok(vec![
                Token::DrawText {
                    x: 0.0,
                    y: 0.0,
                    text: "hello world".to_string(),
                },
                Token::DrawText {
                    x: 10.0,
                    y: 10.0,
                    text: "hello world".to_string(),
                }
            ])
        );
    }
    #[test]
    fn line() {
        let p = parse("DRAW LINE(10,10,20,20)".to_string());
        assert_eq!(
            p,
            Ok(vec![Token::DrawLine {
                x_start: 10.0,
                y_start: 10.0,
                x_end: 20.0,
                y_end: 20.0,
            }])
        );
        let p = parse("DRAW LINE(10.0,10.0,20.0,20.0)".to_string());
        assert_eq!(
            p,
            Ok(vec![Token::DrawLine {
                x_start: 10.0,
                y_start: 10.0,
                x_end: 20.0,
                y_end: 20.0,
            }])
        );
    }
}
