use crate::cst;
use crate::lexer;
use crate::token;
use std::collections::HashMap;

pub struct Parser {
    tokens: Vec<token::Token>,
    position: usize,
    leading_comment_indices: Vec<usize>,
    following_comment_indices: Vec<usize>,
}

impl Parser {
    pub fn new(mut l: lexer::Lexer) -> Parser {
        let mut tokens = Vec::new();
        let mut token = l.next_token();
        while !token.is_none() {
            tokens.push(token.unwrap());
            token = l.next_token();
        }
        let mut p = Parser {
            tokens,
            position: 0,
            leading_comment_indices: Vec::new(),
            following_comment_indices: Vec::new(),
        };
        while p.tokens[p.position].is_comment() {
            p.leading_comment_indices.push(p.position);
            p.position += 1;
        }
        let mut following_comment_idx = p.position + 1;
        while p.tokens[following_comment_idx].is_comment() {
            p.following_comment_indices.push(following_comment_idx);
            following_comment_idx += 1;
        }
        p
    }
    fn get_offset_index(&self, offset: usize) -> usize {
        if offset == 0 {
            return self.position;
        }
        let mut cnt = 0;
        let mut idx = self.position + 1;
        while cnt < offset && idx < self.tokens.len() {
            while self.tokens[idx].is_comment() {
                idx += 1;
            }
            cnt += 1;
            if cnt < offset {
                idx += 1;
            }
        }
        idx
    }
    fn next_token(&mut self) {
        // leading comments
        self.leading_comment_indices = Vec::new();
        let idx = self.get_offset_index(1);
        let from_idx = match self.following_comment_indices.iter().rev().next() {
            Some(n) => *n,
            None => self.position,
        };
        for i in from_idx + 1..idx {
            self.leading_comment_indices.push(i);
        }
        self.position = idx;
        // following comments
        self.following_comment_indices = Vec::new();
        let mut following_comment_idx = self.position + 1;
        while following_comment_idx < self.tokens.len()
            && self.tokens[following_comment_idx].is_comment()
            && self.get_token(0).line == self.tokens[following_comment_idx].line
        {
            self.following_comment_indices.push(following_comment_idx);
            following_comment_idx += 1;
        }
    }
    fn get_token(&self, offset: usize) -> token::Token {
        let idx = self.get_offset_index(offset);
        if idx <= self.tokens.len() - 1 {
            return self.tokens[idx].clone();
        }
        token::Token::new(usize::MAX, usize::MAX, "") // eof token
    }
    fn is_eof(&self, offset: usize) -> bool {
        let idx = self.get_offset_index(offset);
        self.tokens.len() <= idx
    }
    pub fn parse_code(&mut self) -> Vec<cst::Node> {
        let mut code: Vec<cst::Node> = Vec::new();
        while !self.is_eof(0) {
            let stmt = self.parse_statement();
            code.push(stmt);
            self.next_token();
        }
        code
    }
    fn construct_node(&self) -> cst::Node {
        let mut node = cst::Node::new(self.get_token(0).clone());
        // leading comments
        let mut leading_comment_nodes = Vec::new();
        for idx in &self.leading_comment_indices {
            leading_comment_nodes.push(cst::Node::new(self.tokens[*idx].clone()))
        }
        if 0 < leading_comment_nodes.len() {
            node.push_node_vec("leading_comments", leading_comment_nodes);
        }
        // following comments
        let mut following_comment_nodes = Vec::new();
        for idx in &self.following_comment_indices {
            following_comment_nodes.push(cst::Node::new(self.tokens[*idx].clone()))
        }
        if 0 < following_comment_nodes.len() {
            node.push_node_vec("following_comments", following_comment_nodes);
        }
        node
    }
    fn parse_statement(&mut self) -> cst::Node {
        let node = match self.get_token(0).literal.to_uppercase().as_str() {
            "WITH" => self.parse_select_statement(true),
            "SELECT" => self.parse_select_statement(true),
            "(" => self.parse_select_statement(true),
            _ => {
                println!("{:?}", self.get_token(0));
                panic!();
            },
        };
        node
    }
    fn parse_select_statement(&mut self, root: bool) -> cst::Node {
        if self.get_token(0).literal.as_str() == "(" {
            let mut node = self.construct_node();
            self.next_token(); // ( -> select
            node.push_node("stmt", self.parse_select_statement(true));
            self.next_token(); // stmt -> )
            node.push_node("rparen", self.construct_node());
            while self.peek_token_in(&vec!["union", "intersect", "except"]) && root {
                self.next_token(); // stmt -> union
                let mut operator = self.construct_node();
                self.next_token(); // union -> distinct
                operator.push_node("distinct", self.construct_node());
                operator.push_node("left", node);
                self.next_token(); // distinct -> stmt
                operator.push_node("right", self.parse_select_statement(false));
                node = operator;
            }
            if self.peek_token_is(";") && root {
                self.next_token(); // expr -> ;
                node.push_node("semicolon", self.construct_node())
            }
            return node;
        }
        if self.get_token(0).literal.to_uppercase().as_str() == "WITH" {
            let mut with = self.construct_node();
            let mut queries = Vec::new();
            while self.get_token(1).literal.to_uppercase().as_str() != "SELECT" {
                self.next_token(); // with -> ident, ) -> ident
                let mut query = self.construct_node();
                self.next_token(); // ident -> as
                query.push_node("as", self.construct_node());
                self.next_token(); // as -> (
                query.push_node("stmt", self.parse_select_statement(true));
                if self.get_token(1).literal.as_str() == "," {
                    self.next_token(); // ) -> ,
                    query.push_node("comma", self.construct_node());
                }
                queries.push(query);
            }
            with.push_node_vec("queries", queries);
            self.next_token(); // ) -> select
            let mut node = self.parse_select_statement(true);
            node.push_node("with", with);
            return node;
        }
        let mut node = self.construct_node(); // select

        // as struct, as value
        if self.get_token(1).literal.to_uppercase().as_str() == "AS" {
            self.next_token(); // select -> as
            let mut as_ = self.construct_node();
            self.next_token(); // as -> struct, value
            as_.push_node("struct_value", self.construct_node());
            node.push_node("as", as_);
        }

        // distinct
        if self.peek_token_in(&vec!["all", "distinct"]) {
            self.next_token(); // select -> all, distinct
            node.push_node("distinct", self.construct_node());
        }
        self.next_token(); // -> expr
        // columns
        node.children.insert(
            "columns".to_string(),
            cst::Children::NodeVec(self.parse_exprs(&vec!["from", ";", "limit", ")", "union", "intersect", "except"])),
        );
        // from
        if self.peek_token_is("FROM") {
            self.next_token(); // expr -> from
            let mut from = self.construct_node();
            self.next_token(); // from -> table
            from.push_node_vec("tables", self.parse_tables(&vec![]));
            node.push_node("from", from);
        }
        // where
        if self.peek_token_is("WHERE") {
            self.next_token(); // expr -> where
            let mut where_ = self.construct_node();
            self.next_token(); // limit -> expr
            where_.push_node(
                "expr",
                self.parse_expr(999, &vec!["group", "having", ";", ","]),
            );
            //self.next_token(); // parse_expr needs next_token()
            node.push_node("where", where_);
        }
        // group by
        if self.peek_token_is("GROUP") {
            self.next_token(); // expr -> group
            let mut groupby = self.construct_node();
            self.next_token(); // group -> by
            groupby.push_node("by", self.construct_node());
            self.next_token(); // by -> expr
            groupby.push_node_vec(
                "columns",
                self.parse_exprs(&vec!["having", "limit", ";", "order"]),
            );
            node.push_node("groupby", groupby);
        }
        // having
        if self.peek_token_is("HAVING") {
            self.next_token(); // expr -> having
            let mut having = self.construct_node();
            self.next_token(); // by -> expr
            having.push_node("expr", self.parse_expr(999, &vec!["LIMIT", ";", "order"]));
            //self.next_token(); // expr -> limit
            node.push_node("having", having);
        }
        // oeder by
        if self.peek_token_is("order") {
            self.next_token(); // expr -> order
            let mut order = self.construct_node();
            self.next_token(); // order -> by
            order.push_node("by", self.construct_node());
            self.next_token(); // by -> expr
            order.push_node_vec("exprs", self.parse_exprs(&vec!["limit", ","]));
            node.push_node("orderby", order);
        }
        // limit
        if self.peek_token_is("LIMIT") {
            self.next_token(); // expr -> limit
            let mut limit = self.construct_node();
            self.next_token(); // limit -> expr
            limit.push_node("expr", self.parse_expr(999, &vec![";", ",", "offset"]));
            if self.get_token(1).literal.to_uppercase().as_str() == "OFFSET" {
                self.next_token(); // expr -> offset
                let mut offset = self.construct_node();
                self.next_token(); // offset -> expr
                offset.push_node("expr", self.parse_expr(999, &vec!["union", "intersect", "except", ";"]));
                limit.push_node("offset", offset);
            }
            node.push_node("limit", limit);
        }
        // union
        while self.peek_token_in(&vec!["union", "intersect", "except"]) && root {
            self.next_token(); // stmt -> union
            let mut operator = self.construct_node();
            self.next_token(); // union -> distinct
            operator.push_node("distinct", self.construct_node());
            operator.push_node("left", node);
            self.next_token(); // distinct -> stmt
            operator.push_node("right", self.parse_select_statement(false));
            node = operator;
            if self.peek_token_is(";") && root {
                self.next_token(); // expr -> ;
                node.push_node("semicolon", self.construct_node())
            }
        }
        // ;
        if self.peek_token_is(";") && root {
            self.next_token(); // expr -> ;
            node.push_node("semicolon", self.construct_node())
        }
        node
    }
    fn cur_token_in(&self, literals: &Vec<&str>) -> bool {
        for l in literals {
            if self.cur_token_is(l) {
                return true;
            };
        }
        false
    }
    fn peek_token_in(&self, literals: &Vec<&str>) -> bool {
        for l in literals {
            if self.peek_token_is(l) {
                return true;
            };
        }
        false
    }
    fn parse_tables(&mut self, until: &Vec<&str>) -> Vec<cst::Node> {
        let mut tables: Vec<cst::Node> = Vec::new();
        while !self.cur_token_in(&vec!["where", "group", "having", "limit", ";"]) && !self.is_eof(0)
        {
            tables.push(self.parse_table());
            if !self.peek_token_in(&vec!["where", "group", "having", "limit", ";"])
                && !self.is_eof(1)
            {
                self.next_token();
            } else {
                return tables;
            }
        }
        tables // maybe not needed
    }
    fn parse_table(&mut self) -> cst::Node {
        // join
        let mut join = if self.cur_token_in(&vec![
            "left", "right", "cross", "inner", ",", "full", "join",
        ]) {
            if self.cur_token_in(&vec!["join", ","]) {
                let join = self.construct_node();
                self.next_token(); // join -> table
                join
            } else {
                let mut type_ = self.construct_node();
                self.next_token(); // type -> outer, type -> join
                if self.cur_token_is("outer") {
                    type_.push_node("outer", self.construct_node());
                    self.next_token(); // outer -> join
                }
                let mut join = self.construct_node();
                join.push_node("type", type_);
                self.next_token(); // join -> table,
                join
            }
        } else {
            cst::Node::new_none()
        };
        // table
        let mut table = self.parse_expr(
            999,
            &vec![
                "where", "group", "having", "limit", ";", "on", ",", "left", "right", "cross",
                "inner", "join",
            ],
        );
        if join.token != None {
            if self.peek_token_is("on") {
                self.next_token(); // `table` -> on
                let mut on = self.construct_node();
                self.next_token(); // on -> expr
                on.push_node(
                    "expr",
                    self.parse_expr(
                        999,
                        &vec![
                            "left", "right", "cross", "inner", ",", "full", "join", "where",
                            "group", "having", ";",
                        ],
                    ),
                );
                //self.next_token(); // parse_expr needs next_token()
                join.push_node("on", on);
            } //else self.cur_token_is("using") {}
            table.push_node("join", join);
        }
        // TODO... using()
        table
    }
    fn parse_exprs(&mut self, until: &Vec<&str>) -> Vec<cst::Node> {
        let mut exprs: Vec<cst::Node> = Vec::new();
        while !self.cur_token_in(until) && !self.is_eof(0) {
            exprs.push(self.parse_expr(999, until));
            if !self.peek_token_in(until) && !self.is_eof(1) {
                self.next_token();
            } else {
                return exprs;
            }
        }
        exprs // maybe not needed
    }
    fn parse_expr(&mut self, precedence: usize, until: &Vec<&str>) -> cst::Node {
        // prefix or literal
        let mut left = self.construct_node();
        match self.get_token(0).literal.to_uppercase().as_str() {
            "*" => {
                match self.get_token(1).literal.to_uppercase().as_str() {
                    "REPLACE" => {
                        self.next_token(); // * -> replace
                        let mut replace = self.construct_node();
                        self.next_token(); // replace -> (
                        let mut group = self.construct_node();
                        let mut exprs = Vec::new();
                        while self.get_token(1).literal.as_str() != ")" {
                            self.next_token(); // ( -> expr, ident -> expr
                            let expr = self.parse_expr(999, &vec![")"]);
                            exprs.push(expr);
                        }
                        self.next_token(); // ident -> )
                        group.push_node("rparen", self.construct_node());
                        group.push_node_vec("exprs", exprs);
                        replace.push_node("group", group);
                        left.push_node("replace", replace);
                    },
                    "EXCEPT" => {
                        self.next_token(); // * -> except
                        let mut except = self.construct_node();
                        self.next_token(); // except -> (
                        let mut group = self.construct_node();
                        self.next_token(); // ( -> exprs
                        group.push_node_vec("columns", self.parse_exprs(&vec![")"]));
                        self.next_token(); // exprs -> )
                        group.push_node("rparen", self.construct_node());
                        except.push_node("group", group);
                        left.push_node("except", except);
                    },
                    _ => (),
                }
            }
            "(" => {
                self.next_token(); // ( -> expr
                let exprs = self.parse_exprs(&vec![")"]);
                if exprs.len() == 1 {
                    left.push_node("expr", exprs[0].clone());
                } else {
                    left.push_node_vec("exprs", exprs);
                }
                //left.push_node("expr", self.parse_expr(999, until));
                self.next_token(); // expr -> )
                left.push_node("rparen", self.construct_node());
            }
            "-" => {
                self.next_token(); // - -> expr
                let right = self.parse_expr(102, until);
                left.push_node("right", right);
            }
            "[" => {
                self.next_token(); // [ -> exprs
                left.push_node_vec("exprs", self.parse_exprs(&vec!["]"]));
                self.next_token(); // exprs -> ]
                left.push_node("rparen", self.construct_node());
            }
            "DATE" => {
                if self.get_token(1).is_string()
                    || self.peek_token_in(&vec!["b", "r", "br", "rb"])
                        && self.get_token(2).is_string()
                {
                    self.next_token(); // date -> 'yyyy-mm-dd'
                    let right = self.parse_expr(001, until);
                    left.push_node("right", right);
                }
            }
            "TIMESTAMP" => {
                if self.get_token(1).is_string()
                    || self.peek_token_in(&vec!["b", "r", "br", "rb"])
                        && self.get_token(2).is_string()
                {
                    self.next_token(); // timestamp -> 'yyyy-mm-dd'
                    let right = self.parse_expr(001, until);
                    left.push_node("right", right);
                }
            }
            "NUMERIC" => {
                if self.get_token(1).is_string()
                    || self.peek_token_in(&vec!["b", "r", "br", "rb"])
                        && self.get_token(2).is_string()
                {
                    self.next_token(); // timestamp -> 'yyyy-mm-dd'
                    let right = self.parse_expr(001, until);
                    left.push_node("right", right);
                }
            }
            "BIGNUMERIC" => {
                if self.get_token(1).is_string()
                    || self.peek_token_in(&vec!["b", "r", "br", "rb"])
                        && self.get_token(2).is_string()
                {
                    self.next_token(); // timestamp -> 'yyyy-mm-dd'
                    let right = self.parse_expr(001, until);
                    left.push_node("right", right);
                }
            }
            "DECIMAL" => {
                if self.get_token(1).is_string()
                    || self.peek_token_in(&vec!["b", "r", "br", "rb"])
                        && self.get_token(2).is_string()
                {
                    self.next_token(); // timestamp -> 'yyyy-mm-dd'
                    let right = self.parse_expr(001, until);
                    left.push_node("right", right);
                }
            }
            "BIGDECIMAL" => {
                if self.get_token(1).is_string()
                    || self.peek_token_in(&vec!["b", "r", "br", "rb"])
                        && self.get_token(2).is_string()
                {
                    self.next_token(); // timestamp -> 'yyyy-mm-dd'
                    let right = self.parse_expr(001, until);
                    left.push_node("right", right);
                }
            }
            "INTERVAL" => {
                self.next_token(); // interval -> expr
                let right = self.parse_expr(001, &vec!["hour", "month", "year"]);
                self.next_token(); // expr -> hour
                left.push_node("date_part", self.construct_node());
                left.push_node("right", right);
            }
            "R" => {
                if self.get_token(1).is_string() {
                    self.next_token(); // R -> 'string'
                    let right = self.parse_expr(001, until);
                    left.push_node("right", right);
                }
            }
            "B" => {
                if self.get_token(1).is_string() {
                    self.next_token(); // R -> 'string'
                    let right = self.parse_expr(001, until);
                    left.push_node("right", right);
                }
            }
            "BR" => {
                if self.get_token(1).is_string() {
                    self.next_token(); // R -> 'string'
                    let right = self.parse_expr(001, until);
                    left.push_node("right", right);
                }
            }
            "RB" => {
                if self.get_token(1).is_string() {
                    self.next_token(); // R -> 'string'
                    let right = self.parse_expr(001, until);
                    left.push_node("right", right);
                }
            }
            "SELECT" => {
                left = self.parse_select_statement(true);
            }
            "NOT" => {
                self.next_token(); // not -> boolean
                let right = self.parse_expr(110, until);
                left.push_node("right", right);
            }
            "ARRAY" => {
                if self.get_token(1).literal.as_str() == "<" {
                    self.next_token(); // ARRAY -> <
                    let mut type_ = self.construct_node();
                    self.next_token(); // < -> type
                    type_.push_node("type", self.parse_type());
                    self.next_token(); // type -> >
                    type_.push_node("rparen", self.construct_node());
                    left.push_node("type_declaration", type_);
                }
                self.next_token(); // ARRAY -> [, > -> [
                let mut right = self.construct_node();
                self.next_token(); // [ -> exprs
                right.push_node_vec("exprs", self.parse_exprs(&vec!["]"]));
                self.next_token(); // exprs -> ]
                right.push_node("rparen", self.construct_node());
                left.push_node("right", right);
            }
            "STRUCT" => {
                if self.get_token(1).literal.as_str() == "<" {
                    self.next_token(); // struct -> <
                    let mut type_ = self.construct_node();
                    let mut type_declarations = Vec::new();
                    self.next_token(); // < -> ident or type
                    while !self.cur_token_is(">") {
                        let mut type_declaration;
                        if !self.peek_token_in(&vec![",", ">", "TYPE", "<"]) {
                            // `is_identifier` is not availabe here,
                            // because `int64` is valid identifier
                            type_declaration = self.construct_node();
                            self.next_token(); // ident -> type
                        } else {
                            type_declaration = cst::Node::new_none();
                        }
                        type_declaration.push_node("type", self.parse_type());
                        self.next_token(); // type -> , or next_declaration
                        if self.cur_token_is(",") {
                            type_declaration.push_node("comma", self.construct_node());
                            self.next_token(); // , -> next_declaration
                        }
                        type_declarations.push(type_declaration);
                    }
                    type_.push_node("rparen", self.construct_node());
                    type_.push_node_vec("declarations", type_declarations);
                    left.push_node("type_declaration", type_);
                }
                self.next_token(); // struct -> (, > -> (
                let mut right = self.construct_node();
                self.next_token(); // ( -> exprs
                right.push_node_vec("exprs", self.parse_exprs(&vec![")"]));
                self.next_token(); // exprs -> )
                right.push_node("rparen", self.construct_node());
                left.push_node("right", right);
            }
            "CASE" => {
                self.next_token(); // case -> expr, case -> when
                if !self.cur_token_is("WHEN") {
                    left.push_node("expr", self.parse_expr(999, &vec!["WHEN"]));
                    self.next_token(); // expr -> when
                }
                let mut arms = Vec::new();
                while !self.cur_token_is("ELSE") {
                    let mut arm = self.construct_node();
                    self.next_token(); // when -> expr
                    arm.push_node("expr", self.parse_expr(999, &vec!["then"]));
                    self.next_token(); // expr -> then
                    arm.push_node("then", self.construct_node());
                    self.next_token(); // then -> result_expr
                    arm.push_node("result", self.parse_expr(999, &vec!["else", "when"]));
                    self.next_token(); // result_expr -> else, result_expr -> when
                    arms.push(arm)
                }
                let mut arm = self.construct_node();
                self.next_token(); // else -> result_expr
                arm.push_node("result", self.construct_node());
                arms.push(arm);
                left.push_node_vec("arms", arms);
                self.next_token(); // result_expr -> end
                left.push_node("end", self.construct_node());
            }
            _ => (),
        };
        // infix
        //println!("next precedence: {} {}, current_precedence: {} {}", self.get_precedence(1), self.get_token(0).literal, precedence, self.get_token(1).literal);
        while !self.peek_token_in(until) && self.get_precedence(1) < precedence {
            // actually, until is not needed
            match self.get_token(1).literal.to_uppercase().as_str() {
                "BETWEEN" => {
                    self.next_token(); // expr -> between
                    let precedence = self.get_precedence(0);
                    let mut between = self.construct_node();
                    between.push_node("left", left);
                    left = between;
                    self.next_token(); // between -> expr1
                    let mut exprs = Vec::new();
                    exprs.push(self.parse_expr(precedence, until));
                    self.next_token(); // expr1 -> and
                    left.push_node("and", self.construct_node());
                    self.next_token(); // and -> expr2
                    exprs.push(self.parse_expr(precedence, until));
                    left.push_node_vec("right", exprs);
                }
                "LIKE" => {
                    self.next_token(); // expr -> like
                    left = self.parse_binary_operator(left, until);
                }
                "." => {
                    self.next_token(); // expr -> .
                    left = self.parse_binary_operator(left, until);
                }
                "+" => {
                    self.next_token(); // expr -> +
                    left = self.parse_binary_operator(left, until);
                }
                "*" => {
                    self.next_token(); // expr -> +
                    left = self.parse_binary_operator(left, until);
                }
                "=" => {
                    self.next_token(); // expr -> =
                    left = self.parse_binary_operator(left, until);
                }
                "AND" => {
                    self.next_token(); // expr -> =
                    left = self.parse_binary_operator(left, until);
                }
                "OR" => {
                    self.next_token(); // expr -> =
                    left = self.parse_binary_operator(left, until);
                }
                "IN" => {
                    self.next_token(); // expr -> in
                    left = self.parse_in_operator(left);
                }
                "[" => {
                    print!("in new arm");
                    self.next_token(); // expr -> [
                    let mut node = self.construct_node();
                    node.push_node("left", left);
                    //let precedence = self.get_precedence(0);
                    self.next_token(); // [ -> index_expr
                    node.push_node("right", self.parse_expr(999, &vec!["]"]));
                    self.next_token(); // index_expr -> ]
                    node.push_node("rparen", self.construct_node());
                    left = node;
                }
                "(" => {
                    self.next_token(); // expr -> (
                    let mut node = self.construct_node();
                    self.next_token(); // ( -> args
                    node.push_node("func", left);
                    if !self.cur_token_is(")") {
                        node.push_node_vec("args", self.parse_exprs(&vec![")"]));
                        self.next_token(); // expr -> )
                    }
                    node.push_node("rparen", self.construct_node());
                    if self.peek_token_is("over") {
                        self.next_token(); // ) -> over
                        let mut over = self.construct_node();
                        if self.peek_token_is("(") {
                            self.next_token(); // over -> (
                            let mut window = self.construct_node();
                            if self.get_token(1).is_identifier() {
                                self.next_token(); // ( -> identifier
                                window.push_node("name", self.construct_node());
                            }
                            if self.peek_token_is("partition") {
                                self.next_token(); // ( -> partition, order, frame
                                let mut partition = self.construct_node();
                                self.next_token(); // partition -> by
                                partition.push_node("by", self.construct_node());
                                self.next_token(); // by -> exprs
                                partition
                                    .push_node_vec("exprs", self.parse_exprs(&vec!["order", ")"]));
                                window.push_node("partition", partition);
                            }
                            if self.peek_token_is("order") {
                                self.next_token(); // ( -> order, expr -> order
                                let mut order = self.construct_node();
                                self.next_token(); // order -> by
                                order.push_node("by", self.construct_node());
                                self.next_token(); // by -> exprs
                                order.push_node_vec(
                                    "exprs",
                                    self.parse_exprs(&vec!["rows", "range", ")"]),
                                );
                                window.push_node("order", order);
                            }
                            if self.peek_token_in(&vec!["range", "rows"]) {
                                self.next_token(); // ( -> rows, expr -> rows
                                let mut frame = self.construct_node();
                                if self.peek_token_is("between") {
                                    // frame between
                                    self.next_token(); // rows -> between
                                    let mut between = self.construct_node();
                                    self.next_token(); // between -> expr
                                    let mut start = self.parse_expr(999, &vec!["preceding"]);
                                    self.next_token(); // expr -> preceding
                                    start.push_node("preceding", self.construct_node());
                                    frame.push_node("start", start);
                                    self.next_token(); // preceding -> and
                                    between.push_node("and", self.construct_node());
                                    frame.push_node("between", between);
                                    self.next_token(); // and -> expr
                                    let mut end = self.parse_expr(999, &vec![")"]);
                                    self.next_token(); // expr -> following
                                    end.push_node("following", self.construct_node());
                                    frame.push_node("end", end);
                                } else {
                                    // frame start
                                    if !self.peek_token_is(")") {
                                        self.next_token(); // rows -> expr
                                        let mut start = self.parse_expr(999, &vec!["preceding"]);
                                        self.next_token(); // expr -> preceding, row
                                        start.push_node("preceding", self.construct_node());
                                        frame.push_node("start", start);
                                    }
                                }
                                window.push_node("frame", frame)
                            }
                            self.next_token(); // -> )
                            window.push_node("rparen", self.construct_node());
                            over.push_node("window", window);
                            node.push_node("over", over);
                        } else {
                            self.next_token(); // over -> identifier
                            over.push_node("window", self.construct_node());
                            node.push_node("over", over);
                        }
                    }
                    left = node;
                }
                "NOT" => {
                    self.next_token(); // expr -> not
                    let not = self.construct_node();
                    self.next_token(); // not -> in, like
                    if self.cur_token_is("in") {
                        left = self.parse_in_operator(left);
                        left.push_node("not", not);
                    } else if self.cur_token_is("like") {
                        left = self.parse_binary_operator(left, until);
                        left.push_node("not", not);
                    } else {
                        panic!();
                    }
                }
                _ => panic!(),
            }
        }
        // alias
        if self.peek_token_is("as") && precedence == 999 {
            self.next_token(); // expr -> as
            let mut as_ = self.construct_node();
            self.next_token(); // as -> alias
            as_.push_node("alias", self.construct_node());
            left.push_node("as", as_);
        }
        if self.get_token(1).is_identifier() && !self.is_eof(1) && precedence == 999 {
            self.next_token(); // expr -> alias
            let mut as_ = cst::Node {
                token: None,
                children: HashMap::new(),
            };
            as_.push_node("alias", self.construct_node());
            left.push_node("as", as_);
        }
        if self.peek_token_in(&vec!["asc", "desc"]) {
            self.next_token(); // expr -> asc
            left.push_node("order", self.construct_node());
        }
        if self.peek_token_is(",") && precedence == 999 {
            self.next_token(); // expr -> ,
            left.children.insert(
                "comma".to_string(),
                cst::Children::Node(self.construct_node()),
            );
        }
        //self.next_token(); // expr -> from, ',' -> expr
        left
    }
    fn parse_binary_operator(&mut self, left: cst::Node, until: &Vec<&str>) -> cst::Node {
        let precedence = self.get_precedence(0);
        let mut node = self.construct_node();
        self.next_token(); // binary_operator -> expr
        node.push_node("left", left);
        node.push_node("right", self.parse_expr(precedence, until));
        node
    }
    fn parse_in_operator(&mut self, mut left: cst::Node) -> cst::Node {
        let mut node = self.construct_node();
        self.next_token(); // in -> (
        node.push_node("left", left);
        let mut right = self.construct_node();
        self.next_token(); // ( -> expr
        right.push_node_vec("exprs", self.parse_exprs(&vec![")"]));
        self.next_token(); // expr -> )
        right.push_node("rparen", self.construct_node());
        node.push_node("right", right);
        node
    }
    fn peek_token_is(&self, s: &str) -> bool {
        self.get_token(1).literal.to_uppercase() == s.to_uppercase()
    }
    fn cur_token_is(&self, s: &str) -> bool {
        self.get_token(0).literal.to_uppercase() == s.to_uppercase()
    }
    fn parse_type(&mut self) -> cst::Node {
        let res = match self.get_token(0).literal.to_uppercase().as_str() {
            "ARRAY" => {
                let mut res = self.construct_node();
                if self.get_token(1).literal.as_str() == "<" {
                    self.next_token(); // array -> <
                    let mut type_ = self.construct_node();
                    self.next_token(); // < -> type_expr
                    type_.push_node("type", self.parse_type());
                    self.next_token(); // type_expr -> >
                    type_.push_node("rparen", self.construct_node());
                    res.push_node("type_declaration", type_);
                }
                res
            }
            "STRUCT" => {
                let mut res = self.construct_node();
                if self.get_token(1).literal.as_str() == "<" {
                    self.next_token(); // array -> <
                    let mut type_ = self.construct_node();
                    self.next_token(); // < -> type or ident
                    let mut type_declarations = Vec::new();
                    while !self.cur_token_is(">") {
                        let mut type_declaration;
                        if !self.peek_token_in(&vec![",", ">", "TYPE", "<"]) {
                            // `is_identifier` is not availabe here,
                            // because `int64` is valid identifier
                            type_declaration = self.construct_node();
                            self.next_token(); // ident -> type
                        } else {
                            type_declaration = cst::Node::new_none();
                        }
                        type_declaration.push_node("type", self.parse_type());
                        self.next_token(); // type -> , or next_declaration
                        if self.cur_token_is(",") {
                            type_declaration.push_node("comma", self.construct_node());
                            self.next_token(); // , -> next_declaration
                        }
                        type_declarations.push(type_declaration);
                    }
                    type_.push_node("rparen", self.construct_node());
                    type_.push_node_vec("declarations", type_declarations);
                    res.push_node("type_declaration", type_);
                }
                res
            }
            "ANY" => {
                let mut res = self.construct_node();
                self.next_token(); // ANY -> TYPE
                res.push_node("type", self.construct_node());
                res
            }
            _ => self.construct_node(),
        };
        res
    }
    fn get_precedence(&self, offset: usize) -> usize {
        // precedenc
        // https://cloud.google.com/bigquery/docs/reference/standard-sql/operators
        // 001... date, timestamp, r'', b'' (literal)
        // 005... ( (call expression)
        // 101... [], .
        // 102... +, - , ~ (unary operator)
        // 103... *, / , ||
        // 104... +, - (binary operator)
        // 105... <<, >>
        // 106... & (bit operator)
        // 107... ^ (bit operator)
        // 108... | (bit operator)
        // 109... =, <, >, (not)like, between, (not)in
        // 110... not
        // 111... and
        // 112... or
        // 999... LOWEST
        match self.get_token(offset).literal.to_uppercase().as_str() {
            "(" => 005,
            "." => 101,
            "[" => 101,
            "-" => 104,
            "+" => 104,
            "*" => 103,
            "/" => 103,
            "||" => 103,
            "IN" => 109,
            "LIKE" => 109,
            "BETWEEN" => 109,
            "=" => 109,
            "NOT" => {
                match self.get_token(offset + 1).literal.to_uppercase().as_str() {
                    "IN" => {
                        return 109;
                    }
                    "LIKE" => {
                        return 109;
                    }
                    _ => (),
                }
                110
            }
            "AND" => 111,
            "OR" => 112,
            _ => 999,
        }
    }
}

#[cfg(test)]
mod tests;
