use super::*;

#[test]
fn test_parse_code_dml() {
    let test_cases = vec![
        // ----- INSERT statement -----
        TestCase::new(
            "\
INSERT INTO TABLE VALUES(1,2);
",
            "\
self: INSERT (InsertStatement)
input:
  self: VALUES (KeywordWithExprs)
  exprs:
  - self: ( (GroupedExprs)
    exprs:
    - self: 1 (NumericLiteral)
      comma:
        self: , (Symbol)
    - self: 2 (NumericLiteral)
    rparen:
      self: ) (Symbol)
into:
  self: INTO (Keyword)
semicolon:
  self: ; (Symbol)
target_name:
  self: TABLE (Identifier)
",
        ),
        TestCase::new(
            "\
INSERT table_name (col) VALUES(1),(2);
",
            "\
self: INSERT (InsertStatement)
columns:
  self: ( (GroupedExprs)
  exprs:
  - self: col (Identifier)
  rparen:
    self: ) (Symbol)
input:
  self: VALUES (KeywordWithExprs)
  exprs:
  - self: ( (GroupedExprs)
    comma:
      self: , (Symbol)
    exprs:
    - self: 1 (NumericLiteral)
    rparen:
      self: ) (Symbol)
  - self: ( (GroupedExprs)
    exprs:
    - self: 2 (NumericLiteral)
    rparen:
      self: ) (Symbol)
semicolon:
  self: ; (Symbol)
target_name:
  self: table_name (Identifier)
",
        ),
        TestCase::new(
            "\
INSERT table_name (col1, col2) SELECT 1, 2;
",
            "\
self: INSERT (InsertStatement)
columns:
  self: ( (GroupedExprs)
  exprs:
  - self: col1 (Identifier)
    comma:
      self: , (Symbol)
  - self: col2 (Identifier)
  rparen:
    self: ) (Symbol)
input:
  self: SELECT (SelectStatement)
  exprs:
  - self: 1 (NumericLiteral)
    comma:
      self: , (Symbol)
  - self: 2 (NumericLiteral)
semicolon:
  self: ; (Symbol)
target_name:
  self: table_name (Identifier)
",
        ),
        // ----- DELETE statement -----
        TestCase::new(
            "\
DELETE table_name WHERE TRUE;
",
            "\
self: DELETE (DeleteStatement)
semicolon:
  self: ; (Symbol)
table_name:
  self: table_name (Identifier)
where:
  self: WHERE (KeywordWithExpr)
  expr:
    self: TRUE (BooleanLiteral)
",
        ),
        TestCase::new(
            "\
DELETE table_name t WHERE TRUE;
",
            "\
self: DELETE (DeleteStatement)
semicolon:
  self: ; (Symbol)
table_name:
  self: table_name (Identifier)
  alias:
    self: t (Identifier)
where:
  self: WHERE (KeywordWithExpr)
  expr:
    self: TRUE (BooleanLiteral)
",
        ),
        TestCase::new(
            "\
DELETE FROM table_name AS t
WHERE NOT EXISTS (SELECT * FROM t WHERE TRUE);
",
            "\
self: DELETE (DeleteStatement)
from:
  self: FROM (Keyword)
semicolon:
  self: ; (Symbol)
table_name:
  self: table_name (Identifier)
  alias:
    self: t (Identifier)
  as:
    self: AS (Keyword)
where:
  self: WHERE (KeywordWithExpr)
  expr:
    self: NOT (UnaryOperator)
    right:
      self: ( (CallingFunction)
      args:
      - self: SELECT (SelectStatement)
        exprs:
        - self: * (Symbol)
        from:
          self: FROM (KeywordWithExpr)
          expr:
            self: t (Identifier)
        where:
          self: WHERE (KeywordWithExpr)
          expr:
            self: TRUE (BooleanLiteral)
      func:
        self: EXISTS (Identifier)
      rparen:
        self: ) (Symbol)
",
        ),
        // ----- TRUNCATE statement -----
        TestCase::new(
            "\
TRUNCATE table_name t;
",
            "\
self: TRUNCATE (TruncateStatement)
semicolon:
  self: ; (Symbol)
table:
  self: table_name (Keyword)
table_name:
  self: t (Identifier)
",
        ),
        // ----- UPDATE statement -----
        TestCase::new(
            "\
UPDATE TABLE t SET
  col1 = 1,
  col2 = 2
WHERE TRUE;
",
            "\
self: UPDATE (UpdateStatement)
semicolon:
  self: ; (Symbol)
set:
  self: SET (KeywordWithExprs)
  exprs:
  - self: = (BinaryOperator)
    comma:
      self: , (Symbol)
    left:
      self: col1 (Identifier)
    right:
      self: 1 (NumericLiteral)
  - self: = (BinaryOperator)
    left:
      self: col2 (Identifier)
    right:
      self: 2 (NumericLiteral)
table_name:
  self: TABLE (Identifier)
  alias:
    self: t (Identifier)
where:
  self: WHERE (KeywordWithExpr)
  expr:
    self: TRUE (BooleanLiteral)
",
        ),
        TestCase::new(
            "\
UPDATE table1 AS one SET
  one.value=two.value
FROM table2 AS two
WHERE one.id = two.id;
",
            "\
self: UPDATE (UpdateStatement)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: table2 (Identifier)
    alias:
      self: two (Identifier)
    as:
      self: AS (Keyword)
semicolon:
  self: ; (Symbol)
set:
  self: SET (KeywordWithExprs)
  exprs:
  - self: = (BinaryOperator)
    left:
      self: . (BinaryOperator)
      left:
        self: one (Identifier)
      right:
        self: value (Identifier)
    right:
      self: . (BinaryOperator)
      left:
        self: two (Identifier)
      right:
        self: value (Identifier)
table_name:
  self: table1 (Identifier)
  alias:
    self: one (Identifier)
  as:
    self: AS (Keyword)
where:
  self: WHERE (KeywordWithExpr)
  expr:
    self: = (BinaryOperator)
    left:
      self: . (BinaryOperator)
      left:
        self: one (Identifier)
      right:
        self: id (Identifier)
    right:
      self: . (BinaryOperator)
      left:
        self: two (Identifier)
      right:
        self: id (Identifier)
",
        ),
        TestCase::new(
            "\
UPDATE t1 SET
  t1.flg = true
FROM t2 INNER JOIN t3 ON t2.id = t3.id
WHERE t1.id = t3.id;
",
            "\
self: UPDATE (UpdateStatement)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: JOIN (JoinOperator)
    join_type:
      self: INNER (Keyword)
    left:
      self: t2 (Identifier)
    on:
      self: ON (OnClause)
      expr:
        self: = (BinaryOperator)
        left:
          self: . (BinaryOperator)
          left:
            self: t2 (Identifier)
          right:
            self: id (Identifier)
        right:
          self: . (BinaryOperator)
          left:
            self: t3 (Identifier)
          right:
            self: id (Identifier)
    right:
      self: t3 (Identifier)
semicolon:
  self: ; (Symbol)
set:
  self: SET (KeywordWithExprs)
  exprs:
  - self: = (BinaryOperator)
    left:
      self: . (BinaryOperator)
      left:
        self: t1 (Identifier)
      right:
        self: flg (Identifier)
    right:
      self: true (BooleanLiteral)
table_name:
  self: t1 (Identifier)
where:
  self: WHERE (KeywordWithExpr)
  expr:
    self: = (BinaryOperator)
    left:
      self: . (BinaryOperator)
      left:
        self: t1 (Identifier)
      right:
        self: id (Identifier)
    right:
      self: . (BinaryOperator)
      left:
        self: t3 (Identifier)
      right:
        self: id (Identifier)
",
        ),
        // ----- MERGE statement -----
        // DELETE
        TestCase::new(
            "\
MERGE t 
USING s ON t.id = s.id
WHEN MATCHED THEN DELETE;
",
            "\
self: MERGE (MergeStatement)
on:
  self: ON (KeywordWithExpr)
  expr:
    self: = (BinaryOperator)
    left:
      self: . (BinaryOperator)
      left:
        self: t (Identifier)
      right:
        self: id (Identifier)
    right:
      self: . (BinaryOperator)
      left:
        self: s (Identifier)
      right:
        self: id (Identifier)
semicolon:
  self: ; (Symbol)
table_name:
  self: t (Identifier)
using:
  self: USING (KeywordWithExpr)
  expr:
    self: s (Identifier)
whens:
- self: WHEN (WhenClause)
  matched:
    self: MATCHED (Keyword)
  then:
    self: THEN (KeywordWithStatement)
    stmt:
      self: DELETE (SingleTokenStatement)
",
        ),
        // INSERT
        TestCase::new(
            "\
MERGE t1 AS t USING t2 AS s ON t.id = s.id
WHEN NOT MATCHED THEN INSERT ROW
WHEN NOT MATCHED BY TARGET THEN
  INSERT (id, value) VALUES (1,2)
",
            "\
self: MERGE (MergeStatement)
on:
  self: ON (KeywordWithExpr)
  expr:
    self: = (BinaryOperator)
    left:
      self: . (BinaryOperator)
      left:
        self: t (Identifier)
      right:
        self: id (Identifier)
    right:
      self: . (BinaryOperator)
      left:
        self: s (Identifier)
      right:
        self: id (Identifier)
table_name:
  self: t1 (Identifier)
  alias:
    self: t (Identifier)
  as:
    self: AS (Keyword)
using:
  self: USING (KeywordWithExpr)
  expr:
    self: t2 (Identifier)
    alias:
      self: s (Identifier)
    as:
      self: AS (Keyword)
whens:
- self: WHEN (WhenClause)
  matched:
    self: MATCHED (Keyword)
  not:
    self: NOT (Keyword)
  then:
    self: THEN (KeywordWithStatement)
    stmt:
      self: INSERT (InsertStatement)
      input:
        self: ROW (Keyword)
- self: WHEN (WhenClause)
  by_target_or_source:
  - self: BY (Keyword)
  - self: TARGET (Keyword)
  matched:
    self: MATCHED (Keyword)
  not:
    self: NOT (Keyword)
  then:
    self: THEN (KeywordWithStatement)
    stmt:
      self: INSERT (InsertStatement)
      columns:
        self: ( (GroupedExprs)
        exprs:
        - self: id (Identifier)
          comma:
            self: , (Symbol)
        - self: value (Identifier)
        rparen:
          self: ) (Symbol)
      input:
        self: VALUES (KeywordWithExprs)
        exprs:
        - self: ( (GroupedExprs)
          exprs:
          - self: 1 (NumericLiteral)
            comma:
              self: , (Symbol)
          - self: 2 (NumericLiteral)
          rparen:
            self: ) (Symbol)
",
        ),
        // UPDATE
        TestCase::new(
            "\
MERGE dataset.t t USING dataset.s AS s ON t.id = s.id
WHEN NOT MATCHED BY SOURCE THEN
  UPDATE SET id = 999
WHEN NOT MATCHED BY SOURCE AND TRUE THEN
  UPDATE SET
    id = 999,
    value=999
",
            "\
self: MERGE (MergeStatement)
on:
  self: ON (KeywordWithExpr)
  expr:
    self: = (BinaryOperator)
    left:
      self: . (BinaryOperator)
      left:
        self: t (Identifier)
      right:
        self: id (Identifier)
    right:
      self: . (BinaryOperator)
      left:
        self: s (Identifier)
      right:
        self: id (Identifier)
table_name:
  self: . (BinaryOperator)
  alias:
    self: t (Identifier)
  left:
    self: dataset (Identifier)
  right:
    self: t (Identifier)
using:
  self: USING (KeywordWithExpr)
  expr:
    self: . (BinaryOperator)
    alias:
      self: s (Identifier)
    as:
      self: AS (Keyword)
    left:
      self: dataset (Identifier)
    right:
      self: s (Identifier)
whens:
- self: WHEN (WhenClause)
  by_target_or_source:
  - self: BY (Keyword)
  - self: SOURCE (Keyword)
  matched:
    self: MATCHED (Keyword)
  not:
    self: NOT (Keyword)
  then:
    self: THEN (KeywordWithStatement)
    stmt:
      self: UPDATE (UpdateStatement)
      set:
        self: SET (KeywordWithExprs)
        exprs:
        - self: = (BinaryOperator)
          left:
            self: id (Identifier)
          right:
            self: 999 (NumericLiteral)
- self: WHEN (WhenClause)
  and:
    self: AND (KeywordWithExpr)
    expr:
      self: TRUE (BooleanLiteral)
  by_target_or_source:
  - self: BY (Keyword)
  - self: SOURCE (Keyword)
  matched:
    self: MATCHED (Keyword)
  not:
    self: NOT (Keyword)
  then:
    self: THEN (KeywordWithStatement)
    stmt:
      self: UPDATE (UpdateStatement)
      set:
        self: SET (KeywordWithExprs)
        exprs:
        - self: = (BinaryOperator)
          comma:
            self: , (Symbol)
          left:
            self: id (Identifier)
          right:
            self: 999 (NumericLiteral)
        - self: = (BinaryOperator)
          left:
            self: value (Identifier)
          right:
            self: 999 (NumericLiteral)
",
        ),
    ];
    for t in test_cases {
        t.test(0);
    }
}

