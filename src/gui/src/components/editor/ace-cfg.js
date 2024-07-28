import ace from "ace-builds";
import "./lc3";

import modeHtml from 'ace-builds/src-noconflict/mode-html?url';
ace.config.setModuleUrl("ace/mode/html", modeHtml);

import modeJs from 'ace-builds/src-noconflict/mode-javascript?url';
ace.config.setModuleUrl("ace/mode/javascript", modeJs);

import modeLess from 'ace-builds/src-noconflict/mode-less?url';
ace.config.setModuleUrl("ace/mode/less", modeLess);

import themeTextmate from 'ace-builds/src-noconflict/theme-textmate?url';
ace.config.setModuleUrl("ace/theme/textmate", themeTextmate);

import themeTwilight from 'ace-builds/src-noconflict/theme-twilight?url';
ace.config.setModuleUrl("ace/theme/twilight", themeTwilight);

import keybindingVim from 'ace-builds/src-noconflict/keybinding-vim?url';
ace.config.setModuleUrl("ace/keyboard/vim", keybindingVim);

import extSearchbox from 'ace-builds/src-noconflict/ext-searchbox?url';
ace.config.setModuleUrl("ace/ext/searchbox", extSearchbox);

import 'ace-builds/src-noconflict/ext-language_tools';
ace.require("ace/ext/language_tools");
