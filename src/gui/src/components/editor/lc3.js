ace.define("ace/mode/lc3_highlight_rules",
  ["require", "exports","module", "ace/lib/oop", "ace/mode/text_highlight_rules"], 
  function(acequire, exports, module) {
    "use strict";

    const oop = acequire("../lib/oop");
    const TextHighlightRules = acequire("./text_highlight_rules").TextHighlightRules;

    const LC3HighlightRules = function() {
      this.$rules = { start: [
        {
          token: 'keyword.control.assembly',
          regex: '\\b(?:add|and|br(n?z?p?)?|jmp|jsr|jsrr|ld|ldi|ldr|lea|not|ret|rti|st|sti|str|trap|getc|out|putc|puts|in|putsp|halt)\\b',
          caseInsensitive: true
        },
        {
          token: 'variable.parameter.register.assembly',
          regex: '\\b(?:R(?:[0-7]))\\b',
          caseInsensitive: true
        },
        {
          token: 'constant.character.decimal.assembly',
          regex: '[ | ,]#-?[0-9]+',
          caseInsensitive: true
        },
        {
          token: 'constant.character.hexadecimal.assembly',
          regex: '\\bx-?[A-F0-9]+\\b',
          caseInsensitive: true
        },
        { token: 'string.assembly', regex: /'([^\\']|\\.)*'/ },
        { token: 'string.assembly', regex: /"([^\\"]|\\.)*"/ },
        { 
          token: 'support.function.directive.assembly',
          regex: '(\.blkw|\.end|\.external|\.fill|\.orig|\.stringz)\\b',
          caseInsensitive: true
        },
        { token: 'comment.assembly', regex: ';(.*)' }
      ]};
      this.normalizeRules();
    };

  LC3HighlightRules.metaData = {
    fileTypes: [ 'asm' ],
    name: 'LC3',
    scopeName: 'source.assembly'
  };

  oop.inherits(LC3HighlightRules, TextHighlightRules);
  exports.LC3HighlightRules = LC3HighlightRules;
});

ace.define("ace/mode/folding/coffee",
  ["require", "exports", "module", "ace/lib/oop", "ace/mode/folding/fold_mode", "ace/range"],
  function(acequire, exports, module) {
    "use strict";

    const oop = acequire("../../lib/oop");
    const BaseFoldMode = acequire("./fold_mode").FoldMode;
    const Range = acequire("../../range").Range;
    const FoldMode = exports.FoldMode = function() {};

    oop.inherits(FoldMode, BaseFoldMode);

    const FoldModePrototype = function() {
      this.getFoldWidgetRange = (session, foldStyle, row) => { return; };
      this.getFoldWidget = (session, foldStyle, row) => { return ""; };
    }
    FoldModePrototype.call(FoldMode.prototype);
});

ace.define("ace/mode/lc3",
  ["require", "exports", "module", "ace/lib/oop", "ace/mode/text", "ace/mode/lc3_highlight_rules", "ace/mode/folding/coffee"],
  function(acequire, exports, module) {
    "use strict";

    const oop = acequire("../lib/oop");
    const TextMode = acequire("./text").Mode;
    const LC3HighlightRules = acequire("./lc3_highlight_rules").LC3HighlightRules;
    const FoldMode = acequire("./folding/coffee").FoldMode;
    const Mode = function() {
      this.HighlightRules = LC3HighlightRules;
      this.foldingRules = new FoldMode();
      this.$behaviour = this.$defaultBehaviour;
    };

    oop.inherits(Mode, TextMode);

    const ModePrototype = function() {
      this.lineCommentStart = ";";
      this.$id = "ace/mode/lc3";
    }
    ModePrototype.call(Mode.prototype);

    exports.Mode = Mode;
});
