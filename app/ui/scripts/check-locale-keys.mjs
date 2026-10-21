import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "src", "locales");
const en = JSON.parse(readFileSync(join(root, "en.json"), "utf8"));
const ptBr = JSON.parse(readFileSync(join(root, "pt-BR.json"), "utf8"));

const enKeys = Object.keys(en).sort();
const ptKeys = Object.keys(ptBr).sort();

if (enKeys.join("\0") !== ptKeys.join("\0")) {
  const onlyEn = enKeys.filter((k) => !ptKeys.includes(k));
  const onlyPt = ptKeys.filter((k) => !enKeys.includes(k));
  console.error("Locale key mismatch between en.json and pt-BR.json");
  if (onlyEn.length) console.error("  only in en:", onlyEn.join(", "));
  if (onlyPt.length) console.error("  only in pt-BR:", onlyPt.join(", "));
  process.exit(1);
}

console.log(`UI i18n OK (${enKeys.length} keys)`);
