pub(super) fn kebab_cased(name: &str) -> String {
  let mut out = String::with_capacity(name.len());

  for (i, c) in name.chars().enumerate() {
    match c {
      '_' => out.push('-'),
      c if c.is_uppercase() => {
        if i != 0 {
          out.push('-');
        }
        out.extend(c.to_lowercase());
      },
      _ => out.push(c),
    }
  }

  out
}
