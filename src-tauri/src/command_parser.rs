#[derive(Debug, Clone)]
pub struct ParseResult {
  pub transformed_text: String,
  pub commands_applied: Vec<String>,
}

pub fn apply_basic_commands(input: &str) -> ParseResult {
  let mut commands_applied = Vec::new();
  let tokens: Vec<String> = input.split_whitespace().map(|t| t.to_string()).collect();
  let mut words = Vec::new();
  let mut i = 0;

  while i < tokens.len() {
    let tok = tokens[i].to_lowercase();
    let next = tokens.get(i + 1).map(|s| s.to_lowercase());

    if tok == "new" && next.as_deref() == Some("line") {
      commands_applied.push("new line".to_string());
      words.push("\n".to_string());
      i += 2;
      continue;
    }

    if tok == "new" && next.as_deref() == Some("paragraph") {
      commands_applied.push("new paragraph".to_string());
      words.push("\n\n".to_string());
      i += 2;
      continue;
    }

    if tok == "question" && next.as_deref() == Some("mark") {
      commands_applied.push("question mark".to_string());
      words.push("?".to_string());
      i += 2;
      continue;
    }

    match tok.as_str() {
      "comma" => {
        commands_applied.push("comma".to_string());
        words.push(",".to_string());
      }
      "period" => {
        commands_applied.push("period".to_string());
        words.push(".".to_string());
      }
      "newline" => {
        commands_applied.push("newline".to_string());
        words.push("\n".to_string());
      }
      _ => words.push(tokens[i].clone()),
    }
    i += 1;
  }

  let mut text = words.join(" ");
  text = text.replace(" ,", ",");
  text = text.replace(" .", ".");
  text = text.replace(" ?", "?");
  text = text.replace(" \n\n ", "\n\n");
  text = text.replace(" \n\n", "\n\n");
  text = text.replace("\n\n ", "\n\n");
  text = text.replace(" \n ", "\n");
  text = text.replace(" \n", "\n");
  text = text.replace("\n ", "\n");

  ParseResult {
    transformed_text: text,
    commands_applied,
  }
}

#[cfg(test)]
mod tests {
  use super::apply_basic_commands;

  #[test]
  fn applies_punctuation_and_newline() {
    let out = apply_basic_commands("hello comma world newline next line period");
    assert_eq!(out.transformed_text, "hello, world\nnext line.");
    assert_eq!(out.commands_applied.len(), 3);
  }

  #[test]
  fn handles_multiword_commands() {
    let out = apply_basic_commands("what time is it question mark new paragraph next");
    assert_eq!(out.transformed_text, "what time is it?\n\nnext");
    assert_eq!(out.commands_applied.len(), 2);
  }
}
