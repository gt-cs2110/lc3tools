/**
 * Author: Toby Salusky
 *
 * Description: Some relatively simple autocomplete suggestions for LC3 assembly! :D
 *
 * Autocomplete Options (store/modules/Settings.js)
 * 'none': no autocompletion
 * 'basic': builtin registers, instructions, aliases, pseudo ops
 * 'full': `basic` functionality plus best-effort label suggestion (more document processing on keystroke required)
 *
 * Issues/Areas for improvement:
 * > do 'full' autocompletion document parsing more efficiently? (not sure how bad of an issue this currently is?)
 *
 */

const regNames = [0, 1, 2, 3, 4, 5, 6, 7].map(num => `R${num}`);
const instrs = [
  "ADD",
  "AND",
  "NOT",
  "BR", // though combinations like BRnzp exist, we only suggest BR, and simply ignore other forms from being captured as labels
  "JMP",
  "JSR",
  "JSRR",
  "LD",
  "LDI",
  "LDR",
  "LEA",
  "ST",
  "STI",
  "STR",
  "TRAP"
];
const instrAliases = ["HALT", "PUTS", "GETC", "OUT", "IN", "RET"];
const pseudoOps = ["orig", "end", "fill", "blkw", "stringz"];
const keywordSet = new Set(
  [...instrs, ...instrAliases, ...regNames, ...pseudoOps].map(s =>
    s.toLowerCase()
  )
);

const instrCompletions = [
  ...instrs.map(name => ({
    value: name,
    score: 10,
    meta: "instruction"
  }))
];

const instrAliasCompletions = [
  ...instrAliases.map(name => ({
    value: name,
    score: 10,
    meta: "alias"
  }))
];

const pseudoOpCompletions = [
  ...pseudoOps.map(name => ({
    value: name,
    score: 10,
    meta: "pseudo-op"
  }))
];

const regCompletions = [
  ...regNames.map(name => ({
    value: name,
    score: 11,
    meta: "register"
  }))
];

function generateCompletions(mode, doc, pos, prefix) {
  if (mode === "none") {
    return [];
  }

  // typing a number
  if (/^\d/.test(prefix)) {
    return [];
  }

  // in a comment
  const currentLine = doc.getLine(pos.row);
  const currentLineUpToCursor = currentLine.substring(0, pos.column);
  if (currentLineUpToCursor.includes(";")) {
    return [];
  }

  let labelsReferenced = new Set();

  if (mode == "full") {
    const documentLines = doc.getAllLines();
    for (const line of documentLines) {
      const nonCommentLine = line.includes(";")
        ? line.substring(0, line.indexOf(";"))
        : line;

      // remove strings (items between quotes) from potential matches
      const nonStringLine = nonCommentLine.replace(/["'].*?["']/g, "");

      const matches = nonStringLine.match(/\w[\d\w]*/g);
      if (matches) {
        for (const match of matches) {
          if (match === prefix) {
            continue;
          } // do not complete with what is currently being typed!

          const lowerMatch = match.toLowerCase();

          if (
            /^x[a-f\d]+$/.test(lowerMatch) || // hex
            /^\d+$/.test(lowerMatch) || // number
            /^brn?z?p?$/.test(lowerMatch) || // BR
            keywordSet.has(lowerMatch) // existing keyword
          ) {
            // none of the above are labels
            continue;
          }

          labelsReferenced.add(match);
        }
      }
    }
  }

  const labelsCompletions = [...labelsReferenced].map(name => ({
    value: name,
    score: 5,
    meta: "label"
  }));

  const currTokens = currentLineUpToCursor.match(/\b\w+\b|\./g);

  // if we are at the beginning of the line
  if (!currTokens) {
    const tokensAfter = currentLine
      .substring(pos.column - 1)
      .match(/^\b\w+\b/g);
    return [
      // if there are no tokens after the cursor, should suggest instructions
      ...(tokensAfter ? [...instrCompletions, ...instrAliasCompletions] : []),
      ...labelsCompletions
    ];
  }

  let firstToken = currTokens[0].toLowerCase();

  // check if first token is label
  if (
    !keywordSet.has(firstToken) &&
    firstToken !== "." &&
    !firstToken.startsWith("br")
  ) {
    // if label is the only token, suggest instructions (labels shouldn't be attached to pseudo-ops)
    if (currTokens.length === 1) {
      return [...instrCompletions, ...instrAliasCompletions];
    }
    // otherwise, reassign firstToken to the "instruction"
    currTokens.shift();
    firstToken = currTokens[0].toLowerCase();
  }

  // if first token is a period, student is trying to write a pseudo-ops
  if (firstToken === ".") {
    return pseudoOpCompletions;
  }

  // if first token is a BR instruction, suggest labels
  if (/^brn?z?p?$/.test(firstToken)) {
    return labelsCompletions;
  }

  // if instruction, suggest registers
  if (instrs.includes(firstToken.toUpperCase())) {
    return [
      ...regCompletions,
      // suggest labels as well for instructions with PC-relative addressing
      ...(["JSR", "LD", "LDI", "ST", "STI", "LEA"].includes(
        firstToken.toUpperCase()
      )
        ? labelsCompletions
        : [])
    ];
  }

  // otherwise, no suggestions
  return [];
}

// modeGetter => returns current autocomplete setting
export const CreateLc3CompletionProvider = modeGetter => ({
  getCompletions: (_editor, session, pos, prefix, callback) => {
    callback(null, generateCompletions(modeGetter(), session.doc, pos, prefix));
  }
});
