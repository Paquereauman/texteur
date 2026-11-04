with open(r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs", "r", encoding="utf-8") as f:
    text = f.read()

target = "fn est_infinitif(mot: &str) -> bool {"
replacement = """fn est_cjk(c: char) -> bool {
    matches!(c as u32,
        0x2E80..=0x2FDF
        | 0x3000..=0x303F
        | 0x3040..=0x30FF
        | 0x31C0..=0x31EF
        | 0x3400..=0x4DBF
        | 0x4E00..=0x9FFF
        | 0xAC00..=0xD7AF
        | 0xF900..=0xFAFF
        | 0xFF00..=0xFFEF
        | 0x20000..=0x2EBEF)
}

fn est_infinitif(mot: &str) -> bool {"""

if "fn est_cjk" not in text:
    text = text.replace(target, replacement, 1)

with open(r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs", "w", encoding="utf-8", newline="\r\n") as f:
    f.write(text)

print("Added est_cjk to grammaire.rs!")
