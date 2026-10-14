module Assoc = Utils_base.Assoc
(** Convenience facade. Lower layers do not depend on this module. *)

module Bool = Utils_base.Bool
module Error = Utils_base.Error
module List_ext = Utils_base.List_ext
module Number = Utils_base.Number
module String_ext = Utils_base.String_ext
module Utf8 = Utils_base.Utf8
module Binding = Utils_names.Binding
module Id = Utils_names.Id
module Identifier = Utils_names.Identifier
module Intern = Utils_names.Intern
module Naming = Utils_names.Naming
module Symbol = Utils_names.Symbol
module Char_literal = Utils_parsing.Char_literal
module Lexer = Utils_parsing.Lexer
module Parser = Utils_parsing.Parser
module Environment_text = Utils_codec.Environment_text
module Json = Utils_codec.Json
module Text = Utils_codec.Text
module Compiler = Utils_runtime.Compiler
module Compiler_text_api = Utils_runtime.Compiler_text_api
module Language = Utils_runtime.Language
module Machine = Utils_runtime.Machine
module Render = Utils_runtime.Render
module Session = Utils_runtime.Session
module Session_json = Utils_runtime.Session_json
