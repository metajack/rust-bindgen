#[derive(PartialEq, Eq)]
enum QuoteState {
    InNone,
    InSingleQuotes,
    InDoubleQuotes,
}

pub fn parse_process_args(s: &str) -> Vec<String> {
    let s = s.trim();
    let mut parts = Vec::new();
    let mut quote_state = QuoteState::InNone;
    let mut positions = vec![0];
    let mut last = ' ';
    for (i, c) in s.chars().chain(" ".chars()).enumerate() {
        match (last, c) {
            // Match \" set has_escaped and skip
            ('\\', '\"') => (),
            // Match \'
            ('\\', '\'') => (),
            // Match \<space>
            // Check we don't escape the final added space
            ('\\', ' ') if i < s.len() => (),
            // Match \\
            ('\\', '\\') => (),
            // Match <any>"
            (_, '\"') if quote_state == QuoteState::InNone => {
                quote_state = QuoteState::InDoubleQuotes;
                positions.push(i);
                positions.push(i + 1);
            }
            (_, '\"') if quote_state == QuoteState::InDoubleQuotes => {
                quote_state = QuoteState::InNone;
                positions.push(i);
                positions.push(i + 1);
            }
            // Match <any>'
            (_, '\'') if quote_state == QuoteState::InNone => {
                quote_state = QuoteState::InSingleQuotes;
                positions.push(i);
                positions.push(i + 1);
            }
            (_, '\'') if quote_state == QuoteState::InSingleQuotes => {
                quote_state = QuoteState::InNone;
                positions.push(i);
                positions.push(i + 1);
            }
            // Match <any><space>
            // If we are at the end of the string close any open quotes
            (_, ' ') if quote_state == QuoteState::InNone || i >= s.len() => {
                {
                    positions.push(i);

                    let starts = positions.iter().enumerate().filter(|&(i, _)| i % 2 == 0);
                    let ends = positions.iter().enumerate().filter(|&(i, _)| i % 2 == 1);

                    let part: Vec<String> = starts.zip(ends)
                                                  .map(|((_, start), (_, end))| {
                                                      s[*start..*end].to_string()
                                                  })
                                                  .collect();

                    let part = part.join("");

                    if part.len() > 0 {
                        // Remove any extra whitespace outside the quotes
                        let part = &part[..].trim();
                        // Replace quoted characters
                        let part = part.replace("\\\"", "\"");
                        let part = part.replace("\\\'", "\'");
                        let part = part.replace("\\ ", " ");
                        let part = part.replace("\\\\", "\\");
                        parts.push(part);
                    }
                }

                positions.clear();
                positions.push(i + 1);
            }
            (_, _) => (),
        }
        last = c;
    }
    parts
}

#[test]
fn test_parse_process_args() {
    assert_eq!(parse_process_args("a b c"), vec!["a", "b", "c"]);
    assert_eq!(parse_process_args("a \"b\" c"), vec!["a", "b", "c"]);
    assert_eq!(parse_process_args("a \'b\' c"), vec!["a", "b", "c"]);
    assert_eq!(parse_process_args("a \"b c\""), vec!["a", "b c"]);
    assert_eq!(parse_process_args("a \'\"b\"\' c"), vec!["a", "\"b\"", "c"]);
    assert_eq!(parse_process_args("a b\\ c"), vec!["a", "b c"]);
    assert_eq!(parse_process_args("a b c\\"), vec!["a", "b", "c\\"]);
}
