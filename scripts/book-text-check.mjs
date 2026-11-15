// Lists runs of words the repository shares with a book's text, so none of
// the book's prose ends up in the app. The book stays outside the repo:
// extract its text from your own copy first, then pass the text file.
//
//   pdftotext -layout "Double Clutch.pdf" book.txt
//   node scripts/book-text-check.mjs book.txt
//
// It prints each shared run of 5 or more words (or the count given after the
// file). Stat headers, rule names, and numbers will match; sentences shouldn't.

import { execSync } from "node:child_process";
import { readFileSync } from "node:fs";

const [bookPath, minWords = "5"] = process.argv.slice(2);
if (!bookPath) {
  console.error("Usage: node scripts/book-text-check.mjs <book.txt> [min words]");
  process.exit(2);
}
const n = Number(minWords);
const words = (text) =>
  text
    .toLowerCase()
    .replace(/-\s*\n\s*/g, "") // words hyphenated across lines
    .replace(/[’']/g, "'")
    .match(/[a-z0-9']+/g) ?? [];

const book = words(readFileSync(bookPath, "latin1"));
const runs = new Set();
for (let i = 0; i + n <= book.length; i++) runs.add(book.slice(i, i + n).join(" "));

const files = execSync("git ls-files", { encoding: "utf8" })
  .split("\n")
  .filter((f) => f && !/\.(png|ico|icns|lock)$|package-lock\.json$|^LICENSE$/.test(f));
const found = new Set();
for (const file of files) {
  const w = words(readFileSync(file, "utf8"));
  let i = 0;
  while (i + n <= w.length) {
    if (!runs.has(w.slice(i, i + n).join(" "))) {
      i++;
      continue;
    }
    let j = i + n;
    while (j < w.length && runs.has(w.slice(j - n + 1, j + 1).join(" "))) j++;
    found.add(`${file}: ${w.slice(i, j).join(" ")}`);
    i = j;
  }
}
console.log([...found].sort().join("\n") || "No shared runs.");
