with open(r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs", "r", encoding="utf-8") as f:
    text = f.read()

idx = text.find("CAS 6")
start = text.rfind("\n", 0, idx)
end = text.find("{", idx) + 1
old_block = text[start:end]

fix_code = """
                if det_lower == "leurs" && (est_infinitif(&nom_lower) || nom_lower.ends_with("nt") || ["a", "ont", "font", "vont", "sont"].contains(&nom_lower.as_str())) {
                    continue;
                }"""

new_block = old_block + fix_code
text = text[:start] + new_block + text[end:]

with open(r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs", "w", encoding="utf-8", newline="\r\n") as f:
    f.write(text)

print("Fix applied successfully to grammaire.rs!")
