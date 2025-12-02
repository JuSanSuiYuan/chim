#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
启语(Chim)编译器 - 语法分析器
"""
from lexer import TokenType, Token

class ASTNode:
    """AST节点基类"""
    def __init__(self, type_, line, column):
        self.type = type_
        self.line = line
        self.column = column
    
    def __repr__(self):
        return f"{self.type}(line={self.line}, column={self.column})"

class Program(ASTNode):
    """程序根节点"""
    def __init__(self, statements):
        super().__init__("Program", 1, 1)
        self.statements = statements
    
    def __repr__(self):
        return f"Program({len(self.statements)} statements)"

class Function(ASTNode):
    """函数定义节点"""
    def __init__(self, name, params, return_type, body, line, column):
        super().__init__("Function", line, column)
        self.name = name
        self.params = params  # 列表 [(name, type), ...]
        self.return_type = return_type
        self.body = body  # 语句列表
    
    def __repr__(self):
        return f"Function({self.name}, params={len(self.params)}, returns={self.return_type})"

class Struct(ASTNode):
    """结构体定义节点"""
    def __init__(self, name, fields, line, column):
        super().__init__("Struct", line, column)
        self.name = name
        self.fields = fields  # 列表 [(name, type), ...]
    
    def __repr__(self):
        return f"Struct({self.name}, fields={len(self.fields)})"

class VariableDeclaration(ASTNode):
    """变量声明节点"""
    def __init__(self, is_mutable, name, var_type, value, line, column):
        super().__init__("VariableDeclaration", line, column)
        self.is_mutable = is_mutable  # True 表示 设/var
        self.name = name
        self.type = var_type
        self.value = value  # 表达式
    
    def __repr__(self):
        mut = "mut" if self.is_mutable else "let"
        return f"VariableDeclaration({mut} {self.name}:{self.type})"

class MatchStatement(ASTNode):
    """匹配语句节点"""
    def __init__(self, expression, cases, line, column):
        super().__init__("MatchStatement", line, column)
        self.expression = expression
        self.cases = cases  # 列表 [(pattern, body), ...]
    
    def __repr__(self):
        return f"MatchStatement({self.expression}, {len(self.cases)} cases)"

class ForLoop(ASTNode):
    """for循环节点"""
    def __init__(self, variable, iterable, body, line, column):
        super().__init__("ForLoop", line, column)
        self.variable = variable
        self.iterable = iterable  # 表达式
        self.body = body  # 语句列表
    
    def __repr__(self):
        return f"ForLoop({self.variable} in {self.iterable})"

class ReturnStatement(ASTNode):
    """返回语句节点"""
    def __init__(self, expression, line, column):
        super().__init__("ReturnStatement", line, column)
        self.expression = expression
    
    def __repr__(self):
        return f"ReturnStatement({self.expression})"

class CallExpression(ASTNode):
    """函数调用表达式"""
    def __init__(self, function, arguments, line, column):
        super().__init__("CallExpression", line, column)
        self.function = function  # 表达式
        self.arguments = arguments  # 表达式列表
    
    def __repr__(self):
        return f"CallExpression({self.function}, {len(self.arguments)} args)"

class MemberAccess(ASTNode):
    """成员访问表达式 a.b"""
    def __init__(self, object_expr, member_name, line, column):
        super().__init__("MemberAccess", line, column)
        self.object = object_expr
        self.member = member_name  # Identifier
    def __repr__(self):
        return f"MemberAccess({self.object}.{self.member})"

class BinaryExpression(ASTNode):
    """二元表达式"""
    def __init__(self, left, operator, right, line, column):
        super().__init__("BinaryExpression", line, column)
        self.left = left
        self.operator = operator
        self.right = right
    
    def __repr__(self):
        return f"BinaryExpression({self.operator} {self.left} {self.right})"

class UnaryExpression(ASTNode):
    """一元表达式"""
    def __init__(self, operator, operand, line, column):
        super().__init__("UnaryExpression", line, column)
        self.operator = operator
        self.operand = operand
    
    def __repr__(self):
        return f"UnaryExpression({self.operator} {self.operand})"

class Identifier(ASTNode):
    """标识符节点"""
    def __init__(self, name, line, column):
        super().__init__("Identifier", line, column)
        self.name = name
    
    def __repr__(self):
        return f"Identifier({self.name})"

class Literal(ASTNode):
    """字面量节点"""
    def __init__(self, value_type, value, line, column):
        super().__init__("Literal", line, column)
        self.value_type = value_type
        self.value = value
    
    def __repr__(self):
        return f"Literal({self.value_type}={self.value})"

class ArrayLiteral(ASTNode):
    """数组字面量节点"""
    def __init__(self, elements, line, column):
        super().__init__("ArrayLiteral", line, column)
        self.elements = elements
    
    def __repr__(self):
        return f"ArrayLiteral({len(self.elements)} elements)"

class MapLiteral(ASTNode):
    """映射字面量节点"""
    def __init__(self, entries, line, column):
        super().__init__("MapLiteral", line, column)
        self.entries = entries  # 列表 [(key, value), ...]
    
    def __repr__(self):
        return f"MapLiteral({len(self.entries)} entries)"

class BreakStatement(ASTNode):
    def __init__(self, line, column):
        super().__init__("BreakStatement", line, column)
    def __repr__(self):
        return "Break"

class ContinueStatement(ASTNode):
    def __init__(self, line, column):
        super().__init__("ContinueStatement", line, column)
    def __repr__(self):
        return "Continue"

class ImportStatement(ASTNode):
    """导入语句"""
    def __init__(self, module_path, alias, line, column):
        super().__init__("ImportStatement", line, column)
        self.module_path = module_path  # e.g., io::serial
        self.alias = alias
    def __repr__(self):
        return f"Import({self.module_path} as {self.alias})"

class GroupBlock(ASTNode):
    def __init__(self, name_ident, body, line, column):
        super().__init__("GroupBlock", line, column)
        self.name = name_ident
        self.body = body
    def __repr__(self):
        return f"Group({self.name})"

class SnapshotExpression(ASTNode):
    """快照表达式: 快照 x 或 snapshot x"""
    def __init__(self, target, line, column):
        super().__init__("SnapshotExpression", line, column)
        self.target = target  # 被快照的表达式
    def __repr__(self):
        return f"Snapshot({self.target})"

class HandleExpression(ASTNode):
    """句柄表达式: 句柄 x 或 handle x"""
    def __init__(self, target, line, column):
        super().__init__("HandleExpression", line, column)
        self.target = target  # 被引用的表达式
    def __repr__(self):
        return f"Handle({self.target})"

class PointerType(ASTNode):
    """指针类型: *T"""
    def __init__(self, pointee_type, line, column):
        super().__init__("PointerType", line, column)
        self.pointee_type = pointee_type  # 指向的类型
    def __repr__(self):
        return f"*{self.pointee_type}"

class AddressOf(ASTNode):
    """取地址表达式: &variable"""
    def __init__(self, target, line, column):
        super().__init__("AddressOf", line, column)
        self.target = target
    def __repr__(self):
        return f"&{self.target}"

class Dereference(ASTNode):
    """解引用表达式: variable.* 或 *variable (支持两种语法)"""
    def __init__(self, target, line, column):
        super().__init__("Dereference", line, column)
        self.target = target
    def __repr__(self):
        return f"*{self.target}"

class InlineAssembly(ASTNode):
    """内联汇编: asm volatile (code : outputs : inputs : clobbers)"""
    def __init__(self, code, is_volatile, outputs, inputs, clobbers, line, column):
        super().__init__("InlineAssembly", line, column)
        self.code = code  # 汇编代码字符串
        self.is_volatile = is_volatile  # 是否标记为 volatile
        self.outputs = outputs or []  # 输出约束列表
        self.inputs = inputs or []    # 输入约束列表
        self.clobbers = clobbers or [] # 破坏约束列表
    def __repr__(self):
        vol = "volatile " if self.is_volatile else ""
        return f"InlineAssembly({vol}{self.code})"

class ArenaCall(ASTNode):
    """Arena分配调用: system::arena()"""
    def __init__(self, line, column):
        super().__init__("ArenaCall", line, column)
    def __repr__(self):
        return "ArenaCall()"

class TuplePattern(ASTNode):
    """元组模式 - 用于match语句"""
    def __init__(self, elements, line, column):
        super().__init__("TuplePattern", line, column)
        self.elements = elements  # 元组中的元素
    def __repr__(self):
        return f"TuplePattern({self.elements})"

class RangePattern(ASTNode):
    """范围模式 - 用于match语句"""
    def __init__(self, start, end, inclusive, line, column):
        super().__init__("RangePattern", line, column)
        self.start = start
        self.end = end
        self.inclusive = inclusive  # True=闭区间, False=半开区间
    def __repr__(self):
        op = "..." if self.inclusive else ".."
        return f"RangePattern({self.start}{op}{self.end})"

class Parser:
    def __init__(self, tokens):
        self.tokens = tokens
        self.current = 0
        self.dialect = "safe"  # 默认安全方言
    
    def peek(self, offset=0):
        pos = self.current + offset
        if pos < len(self.tokens):
            return self.tokens[pos]
        return None
    
    def advance(self):
        self.current += 1
        return self.tokens[self.current - 1]
    
    def consume(self, token_type, message):
        if self.peek() and self.peek().type == token_type:
            return self.advance()
        raise SyntaxError(f"{message} 行 {self.peek().line if self.peek() else '未知'}")
    
    def match(self, *token_types):
        for token_type in token_types:
            if self.peek() and self.peek().type == token_type:
                return self.advance()
        return None
    
    def skip_newlines(self):
        while self.peek() and self.peek().type == TokenType.NEWLINE:
            self.advance()
    
    def parse(self):
        """解析整个程序"""
        # 跳过文件开头的空行
        self.skip_newlines()
        
        # 检查方言声明
        if self.peek() and self.peek().type in (TokenType.SAFE, TokenType.SYSTEM, TokenType.COMPAT, TokenType.DYNAMIC, TokenType.DSL):
            dialect_token = self.advance()
            if dialect_token.type == TokenType.SAFE:
                self.dialect = 'safe'
            elif dialect_token.type == TokenType.SYSTEM:
                self.dialect = 'system'
            elif dialect_token.type == TokenType.COMPAT:
                self.dialect = 'compat'
            elif dialect_token.type == TokenType.DYNAMIC:
                self.dialect = 'dynamic'
            elif dialect_token.type == TokenType.DSL:
                self.dialect = 'dsl'
            self.skip_newlines()
        
        statements = []
        while self.peek() and self.peek().type != TokenType.EOF:
            stmt = self.parse_statement()
            if stmt:
                statements.append(stmt)
        
        return Program(statements)
    
    def parse_statement(self):
        """解析单个语句"""
        self.skip_newlines()
        
        token = self.peek()
        if not token:
            return None
        if token.type == TokenType.DEDENT:
            return None
        
        # 可见性修饰符：跳过并继续解析后续声明
        if token.type == TokenType.PUB:
            self.advance()
            return self.parse_statement()

        # 模块声明与导入：当前编译期忽略为无副作用语句
        if token.type == TokenType.MODULE:
            while self.peek() and self.peek().type not in (TokenType.NEWLINE, TokenType.EOF):
                self.advance()
            return None
        if token.type == TokenType.IMPORT:
            return self.parse_import()
        if token.type == TokenType.EXTERN:
            # 跳过外部函数声明行
            while self.peek() and self.peek().type not in (TokenType.NEWLINE, TokenType.EOF):
                self.advance()
            return None
        if token.type == TokenType.GROUP:
            return self.parse_group()

        # 函数定义
        if token.type == TokenType.FUNC:
            return self.parse_function()
        # 结构体定义
        elif token.type == TokenType.STRUCT:
            return self.parse_struct()
        # 变量声明
        elif token.type in (TokenType.LET, TokenType.VAR, TokenType.MUT):
            return self.parse_variable_declaration()
        # 匹配语句
        elif token.type == TokenType.MATCH:
            return self.parse_match()
        # 对于循环
        elif token.type == TokenType.FOR:
            return self.parse_for_loop()
        elif token.type == TokenType.WHILE and self.dialect == 'compat':
            return self.parse_while_loop()
        elif token.type == TokenType.IF and self.dialect == 'compat':
            return self.parse_if_statement()
        # 返回语句
        elif token.type == TokenType.RETURN:
            return self.parse_return()
        elif token.type == TokenType.BREAK:
            tok = self.advance()
            return BreakStatement(tok.line, tok.column)
        elif token.type == TokenType.CONTINUE:
            tok = self.advance()
            return ContinueStatement(tok.line, tok.column)
        # 内联汇编语句
        elif token.type == TokenType.ASM:
            asm_token = self.advance()
            return self.parse_inline_assembly(asm_token)
        # 表达式语句
        else:
            expr = self.parse_expression()
            if expr:
                return expr
        
        raise SyntaxError(f"意外的语句开始 行 {token.line}")
    
    def parse_function(self):
        """解析函数定义"""
        fn_token = self.advance()
        name = self.consume(TokenType.IDENTIFIER, "函数名应该是标识符")
        
        self.consume(TokenType.LEFT_PAREN, "函数定义需要左括号")
        params = []
        
        # 解析参数列表
        while self.peek() and self.peek().type != TokenType.RIGHT_PAREN:
            param_name = self.consume(TokenType.IDENTIFIER, "参数名应该是标识符")
            self.consume(TokenType.COLON, "参数需要冒号")
            param_type = self.parse_type()
            params.append((param_name.value, param_type))
            
            if self.peek() and self.peek().type == TokenType.COMMA:
                self.advance()
        
        self.consume(TokenType.RIGHT_PAREN, "函数定义需要右括号")
        
        # 解析返回类型
        return_type = None
        if self.peek() and self.peek().type == TokenType.ARROW:
            self.advance()
            return_type = self.parse_type()
        
        # 解析函数体
        self.consume(TokenType.COLON, "函数体前需要冒号")
        self.skip_newlines()
        self.consume(TokenType.INDENT, "函数体需要缩进")
        
        body = []
        while self.peek() and self.peek().type != TokenType.DEDENT:
            stmt = self.parse_statement()
            if stmt:
                body.append(stmt)
        
        self.consume(TokenType.DEDENT, "函数体需要结束")
        
        return Function(name.value, params, return_type, body, fn_token.line, fn_token.column)
    
    def parse_struct(self):
        """解析结构体定义"""
        struct_token = self.advance()
        name = self.consume(TokenType.IDENTIFIER, "结构体名应该是标识符")
        
        self.consume(TokenType.COLON, "结构体定义需要冒号")
        self.consume(TokenType.INDENT, "结构体体需要缩进")
        
        fields = []
        while self.peek() and self.peek().type != TokenType.DEDENT:
            field_name = self.consume(TokenType.IDENTIFIER, "字段名应该是标识符")
            self.consume(TokenType.COLON, "字段需要冒号")
            field_type = self.parse_type()
            fields.append((field_name.value, field_type))
        
        self.consume(TokenType.DEDENT, "结构体定义需要结束")
        
        return Struct(name.value, fields, struct_token.line, struct_token.column)
    
    def parse_variable_declaration(self):
        """解析变量声明"""
        var_token = self.advance()
        is_mutable = var_token.type in (TokenType.VAR, TokenType.MUT)
        
        name = self.consume(TokenType.IDENTIFIER, "变量名应该是标识符")
        
        var_type = None
        if self.peek() and self.peek().type == TokenType.COLON:
            self.advance()
            var_type = self.parse_type()
        
        # 支持 := 和 = 两种赋值方式
        assign_token = self.match(TokenType.ASSIGN, TokenType.COLON_ASSIGN)
        if not assign_token:
            raise SyntaxError(f"变量声明需要赋值符号 行 {self.peek().line if self.peek() else '未知'}")
        
        value = self.parse_expression()
        
        return VariableDeclaration(
            is_mutable, 
            name.value, 
            var_type, 
            value, 
            var_token.line, 
            var_token.column
        )
    
    def parse_match(self):
        """解析匹配语句 - Swift风格增强"""
        match_token = self.advance()
        expression = self.parse_expression()
        
        # 匹配语句使用冒号 (改为与其他语句一致)
        self.consume(TokenType.COLON, "匹配语句需要冒号")
        self.skip_newlines()
        self.consume(TokenType.INDENT, "匹配语句体需要缩进")
        
        cases = []
        
        while self.peek() and self.peek().type != TokenType.DEDENT:
            # 解析案例分支
            if self.peek().type == TokenType.CASE:
                case_token = self.advance()
                
                # 解析模式 (支持元组、范围、值等)
                pattern = self.parse_match_pattern()
                
                # 检查guard条件 (when/当)
                guard = None
                if self.peek() and self.peek().type == TokenType.WHEN:
                    self.advance()  # consume 'when'
                    guard = self.parse_expression()
                
                self.consume(TokenType.ARROW, "模式后需要 ->")
                self.skip_newlines()
                self.consume(TokenType.INDENT, "case体需要缩进")
                
                # 解析case体
                body = []
                while self.peek() and self.peek().type != TokenType.DEDENT:
                    stmt = self.parse_statement()
                    if stmt:
                        body.append(stmt)
                
                self.consume(TokenType.DEDENT, "case体需要结束")
                cases.append((pattern, guard, body))
            
            # 默认分支 (_ ->)
            elif self.peek().type == TokenType.DEFAULT or (self.peek().type == TokenType.IDENTIFIER and self.peek().value == '_'):
                self.advance()  # consume '_' or '默认'
                self.consume(TokenType.ARROW, "默认分支需要 ->")
                self.skip_newlines()
                self.consume(TokenType.INDENT, "默认分支体需要缩进")
                
                body = []
                while self.peek() and self.peek().type != TokenType.DEDENT:
                    stmt = self.parse_statement()
                    if stmt:
                        body.append(stmt)
                
                self.consume(TokenType.DEDENT, "默认分支体需要结束")
                
                # 使用特殊标记表示默认分支
                default_pattern = Literal("DEFAULT", "_", case_token.line if 'case_token' in locals() else match_token.line, case_token.column if 'case_token' in locals() else match_token.column)
                cases.append((default_pattern, None, body))
                break
            else:
                break
        
        self.consume(TokenType.DEDENT, "匹配语句块需要结束")
        return MatchStatement(expression, cases, match_token.line, match_token.column)
    
    def parse_match_pattern(self):
        """解析匹配模式 - 支持元组、范围、值等"""
        # 元组模式: (a, b, c)
        if self.peek() and self.peek().type == TokenType.LEFT_PAREN:
            self.advance()
            elements = []
            while self.peek() and self.peek().type != TokenType.RIGHT_PAREN:
                elem = self.parse_expression()
                elements.append(elem)
                if self.peek() and self.peek().type == TokenType.COMMA:
                    self.advance()
            self.consume(TokenType.RIGHT_PAREN, "元组模式需要闭合")
            # 返回元组模式(使用特殊标记)
            return TuplePattern(elements, self.previous().line, self.previous().column)
        
        # 普通表达式(可能包含范围)
        left = self.parse_primary()
        
        # 检查范围: a..b
        if self.peek() and self.peek().type == TokenType.RANGE_EXCLUSIVE:
            self.advance()
            right = self.parse_primary()
            return RangePattern(left, right, False, left.line, left.column)  # 半开区间
        elif self.peek() and self.peek().type == TokenType.RANGE_INCLUSIVE:
            self.advance()
            right = self.parse_primary()
            return RangePattern(left, right, True, left.line, left.column)  # 闭区间
        
        return left
    
    def parse_for_loop(self):
        """解析for循环"""
        for_token = self.advance()
        variable = self.consume(TokenType.IDENTIFIER, "循环变量应该是标识符")
        self.consume(TokenType.IN, "for循环需要 '在'/in")
        iterable = self.parse_expression()
        
        self.consume(TokenType.COLON, "循环体前需要冒号")
        self.skip_newlines()
        self.consume(TokenType.INDENT, "循环体需要缩进")
        
        body = []
        while self.peek() and self.peek().type != TokenType.DEDENT:
            stmt = self.parse_statement()
            if stmt:
                body.append(stmt)
        
        self.consume(TokenType.DEDENT, "循环体需要结束")
        
        return ForLoop(variable.value, iterable, body, for_token.line, for_token.column)
    
    def parse_return(self):
        """解析返回语句"""
        return_token = self.advance()
        
        expression = None
        if self.peek() and self.peek().type != TokenType.NEWLINE:
            expression = self.parse_expression()
        
        return ReturnStatement(expression, return_token.line, return_token.column)
    
    def parse_expression(self):
        """解析表达式（简化版本）"""
        return self.parse_binary()
    
    def parse_binary(self):
        """解析二元表达式"""
        left = self.parse_unary()
        
        while self.peek() and self.peek().type in (
            TokenType.PLUS, TokenType.MINUS, TokenType.STAR, TokenType.DIVIDE,
            TokenType.MODULO, TokenType.POWER, TokenType.EQ, TokenType.NEQ,
            TokenType.GT, TokenType.LT, TokenType.GEQ, TokenType.LEQ,
            TokenType.AND, TokenType.OR, TokenType.CHAN_SEND
        ):
            operator_token = self.advance()
            # 在二元表达式中，* 是乘法运算符
            operator = operator_token.value if operator_token.type != TokenType.STAR else '*'
            right = self.parse_unary()
            left = BinaryExpression(left, operator, right, left.line, left.column)
        
        return left
    
    def parse_unary(self):
        """解析一元表达式"""
        # 取地址: &variable
        if self.peek() and self.peek().type == TokenType.AMPERSAND:
            op_token = self.advance()
            operand = self.parse_unary()
            return AddressOf(operand, op_token.line, op_token.column)
        
        # 前缀解引用: *variable (类 C 风格)
        if self.peek() and self.peek().type == TokenType.STAR:
            op_token = self.advance()
            operand = self.parse_unary()
            return Dereference(operand, op_token.line, op_token.column)
        
        if self.peek() and self.peek().type in (TokenType.NOT, TokenType.MINUS, TokenType.CHAN_SEND):
            operator = self.advance().value
            operand = self.parse_unary()
            return UnaryExpression(operator, operand, operand.line, operand.column)
        
        return self.parse_postfix()
    
    def parse_postfix(self):
        """解析后缀表达式(包括成员访问和解引用)"""
        expr = self.parse_primary()
        
        while True:
            # 成员访问: expr.member
            if self.peek() and self.peek().type == TokenType.DOT:
                dot_token = self.advance()
                # 检查是否是 .* (解引用)
                if self.peek() and self.peek().type == TokenType.STAR:
                    self.advance()  # consume *
                    expr = Dereference(expr, dot_token.line, dot_token.column)
                else:
                    member_tok = self.consume(TokenType.IDENTIFIER, "成员名应该是标识符")
                    expr = MemberAccess(expr, Identifier(member_tok.value, member_tok.line, member_tok.column), dot_token.line, dot_token.column)
            # 函数调用 / 括号构造
            elif self.peek() and self.peek().type == TokenType.LEFT_PAREN:
                # 检查是否是特殊构造函数
                if isinstance(expr, Identifier):
                    if expr.name in ("数组", "array"):
                        expr = self.parse_array_paren(expr.line, expr.column)
                        continue
                    elif expr.name in ("映射", "map"):
                        expr = self.parse_map_paren(expr.line, expr.column)
                        continue
                # 普通函数调用
                expr = self.parse_call(expr)
            else:
                break
        
        return expr
    
    def parse_primary(self):
        """解析基本表达式"""
        token = self.advance()
        
        # 内联汇编: asm 或 asm volatile
        if token.type == TokenType.ASM:
            return self.parse_inline_assembly(token)
        
        if token.type == TokenType.IDENTIFIER:
            ident = Identifier(token.value, token.line, token.column)
            return ident
        elif token.type == TokenType.INTEGER:
            return Literal("INTEGER", token.value, token.line, token.column)
        
        elif token.type == TokenType.FLOAT:
            return Literal("FLOAT", token.value, token.line, token.column)
        
        elif token.type == TokenType.STRING:
            return Literal("STRING", token.value, token.line, token.column)
        
        elif token.type == TokenType.CHAR:
            return Literal("CHAR", token.value, token.line, token.column)
        
        elif token.type == TokenType.TRUE:
            return Literal("BOOLEAN", True, token.line, token.column)
        
        elif token.type == TokenType.FALSE:
            return Literal("BOOLEAN", False, token.line, token.column)
        
        elif token.type == TokenType.NIL:
            return Literal("NIL", None, token.line, token.column)

        elif token.type == TokenType.LEFT_PAREN:
            expr = self.parse_expression()
            self.consume(TokenType.RIGHT_PAREN, "未闭合的括号")
            return expr
            
        elif token.type == TokenType.LEFT_BRACKET:
            # 解析数组字面量
            return self.parse_array_literal(token.line, token.column)
            
        elif token.type == TokenType.LEFT_BRACE:
            # 解析映射字面量
            return self.parse_map_literal(token.line, token.column)
        
        # 快照表达式: 快照 x 或 snapshot x
        elif token.type == TokenType.SNAPSHOT:
            target = self.parse_primary()
            return SnapshotExpression(target, token.line, token.column)
        
        # 句柄表达式: 句柄 x 或 handle x
        elif token.type == TokenType.HANDLE:
            target = self.parse_primary()
            return HandleExpression(target, token.line, token.column)

        raise SyntaxError(f"意外的表达式 行 {token.line}")

    def parse_import(self):
        tok = self.advance()  # IMPORT
        parts = []
        # 读取模块路径 a::b::c
        while self.peek() and self.peek().type == TokenType.IDENTIFIER:
            parts.append(self.advance().value)
            if self.peek() and self.peek().type == TokenType.SCOPE:
                self.advance()
                continue
            else:
                break
        alias = None
        if self.peek() and self.peek().type == TokenType.AS:
            self.advance()
            alias_tok = self.consume(TokenType.IDENTIFIER, "导入别名需要标识符")
            alias = alias_tok.value
        # 跳到行尾
        while self.peek() and self.peek().type not in (TokenType.NEWLINE, TokenType.EOF):
            self.advance()
        return ImportStatement("::".join(parts), alias, tok.line, tok.column)

    def parse_group(self):
        tok = self.advance()  # GROUP
        name_tok = self.consume(TokenType.IDENTIFIER, "组需要名称")
        self.consume(TokenType.COLON, "组块需要冒号")
        self.skip_newlines()
        self.consume(TokenType.INDENT, "组块需要缩进")
        body = []
        while self.peek() and self.peek().type != TokenType.DEDENT:
            stmt = self.parse_statement()
            if stmt:
                body.append(stmt)
        self.consume(TokenType.DEDENT, "组块需要结束")
        return GroupBlock(Identifier(name_tok.value, name_tok.line, name_tok.column), body, tok.line, tok.column)
    
    def parse_call(self, function):
        """解析函数调用"""
        self.consume(TokenType.LEFT_PAREN, "函数调用需要左括号")
        
        arguments = []
        while self.peek() and self.peek().type != TokenType.RIGHT_PAREN:
            arg = self.parse_expression()
            arguments.append(arg)
            
            if self.peek() and self.peek().type == TokenType.COMMA:
                self.advance()
        
        self.consume(TokenType.RIGHT_PAREN, "函数调用需要右括号")
        
        return CallExpression(function, arguments, function.line, function.column)
    
    def parse_type(self):
        """解析类型表达式"""
        token = self.peek()
        
        # 指针类型: *T
        if token and token.type == TokenType.STAR:
            self.advance()
            pointee_type = self.parse_type()
            return f"*{pointee_type}"

        # 基本类型
        if token and token.type == TokenType.IDENTIFIER:
            # 检查是否是泛型类型
            if self.peek(1) and self.peek(1).type == TokenType.LEFT_BRACKET:
                self.advance()  # 消费类型名
                self.advance()  # 消费左括号

                type_args = []
                while self.peek() and self.peek().type != TokenType.RIGHT_BRACKET:
                    type_arg = self.parse_type()
                    type_args.append(type_arg)

                    if self.peek() and self.peek().type == TokenType.COMMA:
                        self.advance()

                self.consume(TokenType.RIGHT_BRACKET, "泛型类型需要右括号")
                return f"{token.value}<{', '.join(str(t) for t in type_args)}>"
            else:
                self.advance()
                return token.value

        # 下划线类型推断
        elif token and token.type == TokenType.DEFAULT:
            self.advance()
            return '_'

        raise SyntaxError(f"意外的类型 行 {token.line}")
        
    def parse_array_literal(self, line, column):
        """解析数组字面量 [expr1, expr2, ...]"""
        elements = []
        
        while self.peek() and self.peek().type != TokenType.RIGHT_BRACKET:
            element = self.parse_expression()
            elements.append(element)
            
            if self.peek() and self.peek().type == TokenType.COMMA:
                self.advance()
        
        self.consume(TokenType.RIGHT_BRACKET, "未闭合的数组括号")
        return ArrayLiteral(elements, line, column)
        
    def parse_map_literal(self, line, column):
        """解析映射字面量 {key1: value1, key2: value2, ...}"""
        entries = []
        
        while self.peek() and self.peek().type != TokenType.RIGHT_BRACE:
            key = self.parse_expression()
            self.consume(TokenType.COLON, "映射键值对需要冒号")
            value = self.parse_expression()
            entries.append((key, value))
            
            if self.peek() and self.peek().type == TokenType.COMMA:
                self.advance()
        
        self.consume(TokenType.RIGHT_BRACE, "未闭合的映射括号")
        return MapLiteral(entries, line, column)

    def parse_array_paren(self, line, column):
        elements = []
        self.consume(TokenType.LEFT_PAREN, "数组需要左括号")
        while self.peek() and self.peek().type != TokenType.RIGHT_PAREN:
            elements.append(self.parse_expression())
            if self.peek() and self.peek().type == TokenType.COMMA:
                self.advance()
        self.consume(TokenType.RIGHT_PAREN, "数组需要右括号")
        return ArrayLiteral(elements, line, column)

    def parse_map_paren(self, line, column):
        entries = []
        self.consume(TokenType.LEFT_PAREN, "映射需要左括号")
        while self.peek() and self.peek().type != TokenType.RIGHT_PAREN:
            key_tok = self.consume(TokenType.IDENTIFIER, "映射键需要标识符")
            self.consume(TokenType.COLON, "映射键值对需要冒号")
            value = self.parse_expression()
            entries.append((Identifier(key_tok.value, key_tok.line, key_tok.column), value))
            if self.peek() and self.peek().type == TokenType.COMMA:
                self.advance()
        self.consume(TokenType.RIGHT_PAREN, "映射需要右括号")
        return MapLiteral(entries, line, column)
    def parse_while_loop(self):
        tok = self.advance()
        condition = self.parse_expression()
        self.consume(TokenType.COLON, "循环体前需要冒号")
        self.skip_newlines()
        self.consume(TokenType.INDENT, "循环体需要缩进")
        body = []
        while self.peek() and self.peek().type != TokenType.DEDENT:
            stmt = self.parse_statement()
            if stmt:
                body.append(stmt)
        self.consume(TokenType.DEDENT, "循环体需要结束")
        true_lit = Literal("BOOLEAN", True, tok.line, tok.column)
        default_body = []
        return MatchStatement(condition, [(true_lit, body), (Identifier('_', tok.line, tok.column), default_body)], tok.line, tok.column)

    def parse_if_statement(self):
        tok = self.advance()
        branches = []
        cond = self.parse_expression()
        self.consume(TokenType.COLON, "if 需要冒号")
        self.skip_newlines()
        self.consume(TokenType.INDENT, "if 体需要缩进")
        body = []
        while self.peek() and self.peek().type != TokenType.DEDENT:
            stmt = self.parse_statement()
            if stmt:
                body.append(stmt)
        self.consume(TokenType.DEDENT, "if 体需要结束")
        branches.append((cond, body))
        while self.peek() and self.peek().type == TokenType.ELIF:
            self.advance()
            econd = self.parse_expression()
            self.consume(TokenType.COLON, "elif 需要冒号")
            self.skip_newlines()
            self.consume(TokenType.INDENT, "elif 体需要缩进")
            ebody = []
            while self.peek() and self.peek().type != TokenType.DEDENT:
                stmt = self.parse_statement()
                if stmt:
                    ebody.append(stmt)
            self.consume(TokenType.DEDENT, "elif 体需要结束")
            branches.append((econd, ebody))
        else_body = []
        if self.peek() and self.peek().type == TokenType.ELSE:
            self.advance()
            self.consume(TokenType.COLON, "else 需要冒号")
            self.skip_newlines()
            self.consume(TokenType.INDENT, "else 体需要缩进")
            while self.peek() and self.peek().type != TokenType.DEDENT:
                stmt = self.parse_statement()
                if stmt:
                    else_body.append(stmt)
            self.consume(TokenType.DEDENT, "else 体需要结束")
        def lower_chain(idx: int):
            cond_i, body_i = branches[idx]
            true_lit = Literal("BOOLEAN", True, tok.line, tok.column)
            if idx == len(branches) - 1:
                default_branch_body = else_body
            else:
                nested = lower_chain(idx + 1)
                default_branch_body = [nested]
            return MatchStatement(cond_i, [(true_lit, body_i), (Identifier('_', tok.line, tok.column), default_branch_body)], tok.line, tok.column)
        return lower_chain(0)
    
    def parse_inline_assembly(self, asm_token):
        """解析内联汇编: asm [volatile] (code : outputs : inputs : clobbers)"""
        is_volatile = False
        
        # 检查 volatile 关键字
        if self.peek() and self.peek().type == TokenType.VOLATILE:
            self.advance()
            is_volatile = True
        
        # 必须有左括号
        self.consume(TokenType.LEFT_PAREN, "内联汇编需要左括号")
        
        # 解析汇编代码字符串
        code_token = self.consume(TokenType.STRING, "内联汇编代码必须是字符串")
        code = code_token.value
        
        outputs = []
        inputs = []
        clobbers = []
        
        # 解析约束 (简化版，仅解析基本格式)
        # 完整格式: (code : output_constraints : input_constraints : clobbers)
        
        if self.peek() and self.peek().type == TokenType.COLON:
            self.advance()  # consume ':'
            # 解析输出约束 (简化: 跳过到下一个冒号或右括号)
            while self.peek() and self.peek().type not in (TokenType.COLON, TokenType.RIGHT_PAREN):
                # 跳过约束内容 (简化处理)
                self.advance()
        
        if self.peek() and self.peek().type == TokenType.COLON:
            self.advance()  # consume ':'
            # 解析输入约束
            while self.peek() and self.peek().type not in (TokenType.COLON, TokenType.RIGHT_PAREN):
                self.advance()
        
        if self.peek() and self.peek().type == TokenType.COLON:
            self.advance()  # consume ':'
            # 解析 clobbers
            while self.peek() and self.peek().type != TokenType.RIGHT_PAREN:
                self.advance()
        
        self.consume(TokenType.RIGHT_PAREN, "内联汇编需要右括号")
        
        return InlineAssembly(code, is_volatile, outputs, inputs, clobbers, asm_token.line, asm_token.column)
    
    def previous(self):
        """返回上一个 token"""
        if self.current > 0:
            return self.tokens[self.current - 1]
        return None
