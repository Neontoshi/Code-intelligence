pub struct SymbolTable {
    pub replacements: Vec<(&'static str, &'static str)>,
}

impl SymbolTable {
    pub fn universal() -> Self {
        SymbolTable {
            replacements: vec![
                ("function ", "ƒ "), ("def ", "ƒ "), ("fn ", "ƒ "), ("func ", "ƒ "),
                ("class ", "∁ "), ("struct ", "∁ "), ("impl ", "⚡ "),
                ("return ", "⇐ "), ("import ", "↓ "), ("from ", "← "), ("export ", "↑ "),
                ("const ", "⊲ "), ("let ", "⊲ "), ("var ", "⊲ "), ("mut ", "♲ "),
                ("async ", "∂ "), ("await ", "∫ "),
                ("public ", "+ "), ("private ", "- "), ("protected ", "~ "),
                ("static ", "§ "), ("abstract ", "Δ "), ("interface ", "⊡ "),
                ("extends ", "→ "), ("implements ", "⊢ "),
                ("this.", "@"), ("self.", "@"), ("super.", "⋉"),
                ("new ", "★ "), ("throw ", "↑ "), ("catch ", "⚠ "),
                ("if ", "? "), ("else ", ": "), ("for ", "∀ "), ("while ", "∃ "),
                ("match ", "≋ "), ("switch ", "≋ "), ("case ", "▸ "),
                ("true", "⊤"), ("false", "⊥"), ("null", "∅"), ("None", "∅"),
                ("Option<", "?<"), ("Result<", "!<"), ("Vec<", "□<"),
                ("->", "→"), ("=>", "⇒"), ("::", "∷"),
                ("&&", "∧"), ("||", "∨"), ("==", "≡"), ("!=", "≠"),
                ("<=", "≤"), (">=", "≥"),
            ],
        }
    }
}
