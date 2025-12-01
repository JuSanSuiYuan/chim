#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Chim语义分析器
- 作用域管理
- 类型检查
- 方言约束检查
- 符号表构建
"""

from parser_ import *
from typing import Dict, List, Optional, Set
from enum import Enum

class Lifetime(Enum):
    """生命周期类型（类June语言）"""
    LOCAL = "Local"          # 不逃离当前函数
    PARAM = "Param"          # 通过参数传递
    RETURN = "Return"        # 作为返回值传递
    GROUP = "Group"          # 属于某个组
    SNAPSHOT = "Snapshot"    # 快照（只讻，不延长源组寿命）
    HANDLE = "Handle"        # 句柄（轻量引用）
    
    def __repr__(self):
        return self.value

class Type:
    """类型表示"""
    def __init__(self, name: str, params: List['Type'] = None):
        self.name = name
        self.params = params or []
    
    def __eq__(self, other):
        if not isinstance(other, Type):
            return False
        return self.name == other.name and self.params == other.params
    
    def __repr__(self):
        if self.params:
            params_str = ', '.join(str(p) for p in self.params)
            return f"{self.name}<{params_str}>"
        return self.name
    
    def is_compatible(self, other: 'Type') -> bool:
        """检查类型兼容性"""
        if self.name == "Any" or other.name == "Any":
            return True
        if self.name != other.name:
            return False
        if len(self.params) != len(other.params):
            return False
        return all(p1.is_compatible(p2) for p1, p2 in zip(self.params, other.params))

# 内置类型
BUILTIN_TYPES = {
    "整数": Type("整数"),
    "浮点数": Type("浮点数"),
    "字符": Type("字符"),
    "字符串": Type("字符串"),
    "布尔": Type("布尔"),
    "单元": Type("单元"),
    "Any": Type("Any"),
    # 英文别名
    "int": Type("整数"),
    "float": Type("浮点数"),
    "char": Type("字符"),
    "string": Type("字符串"),
    "bool": Type("布尔"),
    "unit": Type("单元"),
}

class Symbol:
    """符号表项"""
    def __init__(self, name: str, type_: Type, is_mutable: bool = False, kind: str = "variable", lifetime: Lifetime = None):
        self.name = name
        self.type = type_
        self.is_mutable = is_mutable
        self.kind = kind  # variable, function, struct, etc.
        self.lifetime = lifetime or Lifetime.LOCAL  # 默认LOCAL生命周期

class Scope:
    """作用域"""
    def __init__(self, parent: Optional['Scope'] = None):
        self.parent = parent
        self.symbols: Dict[str, Symbol] = {}
    
    def define(self, symbol: Symbol):
        """定义符号"""
        if symbol.name in self.symbols:
            raise SemanticError(f"符号 '{symbol.name}' 已定义")
        self.symbols[symbol.name] = symbol
    
    def lookup(self, name: str) -> Optional[Symbol]:
        """查找符号"""
        if name in self.symbols:
            return self.symbols[name]
        if self.parent:
            return self.parent.lookup(name)
        return None
    
    def lookup_local(self, name: str) -> Optional[Symbol]:
        """仅在当前作用域查找"""
        return self.symbols.get(name)

class SemanticError(Exception):
    """语义错误"""
    pass

class LifetimeAnalyzer:
    """生命周期分析器 - 实现类June的组生命周期推断"""
    def __init__(self):
        self.current_group = None  # 当前所在的组
        self.group_stack = []      # 组嵌套栈
        self.var_lifetimes = {}    # 变量 -> 生命周期
        self.group_vars = {}       # 组 -> 变量集合
        self.in_function = False
        self.returned_vars = set() # 被返回的变量
    
    def enter_group(self, group_name: str):
        """进入组块"""
        self.group_stack.append(group_name)
        self.current_group = group_name
        self.group_vars[group_name] = set()
    
    def exit_group(self):
        """退出组块"""
        if self.group_stack:
            self.group_stack.pop()
        self.current_group = self.group_stack[-1] if self.group_stack else None
    
    def infer_lifetime(self, var_name: str, is_param: bool = False, is_returned: bool = False) -> Lifetime:
        """推断变量生命周期（类June: Local/Param/Return）"""
        if is_param:
            lifetime = Lifetime.PARAM
        elif is_returned or var_name in self.returned_vars:
            lifetime = Lifetime.RETURN
        elif self.current_group:
            lifetime = Lifetime.GROUP
        else:
            lifetime = Lifetime.LOCAL
        
        self.var_lifetimes[var_name] = lifetime
        
        # 如果在组内，记录到组
        if self.current_group:
            self.group_vars[self.current_group].add(var_name)
        
        return lifetime
    
    def check_cross_group_reference(self, var_name: str, access_from_group: str) -> bool:
        """检查跨组引用是否合法"""
        if var_name not in self.var_lifetimes:
            return True  # 未知变量，跨过
        
        var_lifetime = self.var_lifetimes[var_name]
        
        # 快照和句柄可以跨组传递
        if var_lifetime in (Lifetime.SNAPSHOT, Lifetime.HANDLE):
            return True
        
        # 检查是否属于同一组
        for group, vars_in_group in self.group_vars.items():
            if var_name in vars_in_group:
                if group != access_from_group:
                    # 跨组访问，需要快照/句柄
                    return False
        
        return True
    
    def mark_as_returned(self, var_name: str):
        """标记变量被返回"""
        self.returned_vars.add(var_name)
        if var_name in self.var_lifetimes:
            self.var_lifetimes[var_name] = Lifetime.RETURN

class SemanticAnalyzer:
    """语义分析器"""
    def __init__(self, dialect: str = "safe"):
        self.dialect = dialect  # safe, system, compat
        self.global_scope = Scope()
        self.current_scope = self.global_scope
        self.errors: List[str] = []
        self.current_function_return_type: Optional[Type] = None
        self.in_unsafe_block = False  # 跟踪是否在unsafe块中
        self.borrows: Dict[str, List[str]] = {}  # 跟踪借用关系
        self.lifetime_analyzer = LifetimeAnalyzer()  # 生命周期分析器
        
        # 初始化内置符号
        self._init_builtins()
    
    def _init_builtins(self):
        """初始化内置符号"""
        # 内置函数
        self.global_scope.define(Symbol("输出", Type("函数"), kind="function"))
        self.global_scope.define(Symbol("print", Type("函数"), kind="function"))
    
    def enter_scope(self):
        """进入新作用域"""
        self.current_scope = Scope(self.current_scope)
    
    def exit_scope(self):
        """退出当前作用域"""
        if self.current_scope.parent:
            self.current_scope = self.current_scope.parent
    
    def error(self, msg: str):
        """记录错误"""
        self.errors.append(msg)
    
    def analyze(self, ast) -> bool:
        """分析AST"""
        self.errors.clear()
        
        try:
            # 处理Program对象
            if hasattr(ast, 'statements'):
                statements = ast.statements
            elif isinstance(ast, list):
                statements = ast
            else:
                statements = [ast]
            
            for node in statements:
                self.visit_statement(node)
            return len(self.errors) == 0
        except SemanticError as e:
            self.error(str(e))
            return False
    
    def visit_statement(self, stmt: ASTNode):
        """访问语句"""
        if isinstance(stmt, Function):
            self.visit_function(stmt)
        elif isinstance(stmt, Struct):
            self.visit_struct(stmt)
        elif isinstance(stmt, VariableDeclaration):
            self.visit_variable_declaration(stmt)
        elif isinstance(stmt, MatchStatement):
            self.visit_match_statement(stmt)
        elif isinstance(stmt, ForLoop):
            self.visit_for_loop(stmt)
        elif isinstance(stmt, ReturnStatement):
            self.visit_return_statement(stmt)
        elif isinstance(stmt, GroupBlock):
            self.visit_group_block(stmt)
        elif isinstance(stmt, CallExpression):
            self.visit_expression(stmt)
    
    def visit_function(self, node: Function):
        """访问函数定义"""
        # 解析返回类型
        return_type = self.resolve_type(node.return_type) if node.return_type else Type("单元")
        
        # 定义函数符号
        func_symbol = Symbol(node.name, return_type, kind="function")
        self.global_scope.define(func_symbol)
        
        # 进入函数作用域
        self.enter_scope()
        self.current_function_return_type = return_type
        
        # 定义参数
        for param_name, param_type in node.params:
            param_type_obj = self.resolve_type(param_type)
            self.current_scope.define(Symbol(param_name, param_type_obj))
        
        # 分析函数体
        for stmt in node.body:
            self.visit_statement(stmt)
        
        self.current_function_return_type = None
        self.exit_scope()
    
    def visit_struct(self, node: Struct):
        """访问结构体定义"""
        # 定义结构体类型
        struct_type = Type(node.name)
        struct_symbol = Symbol(node.name, struct_type, kind="struct")
        self.global_scope.define(struct_symbol)
    
    def visit_variable_declaration(self, node: VariableDeclaration):
        """访问变量声明"""
        # 推断类型
        value_type = self.visit_expression(node.value)
        
        # 解析声明的类型
        if node.type:
            declared_type = self.resolve_type(node.type)
            if not value_type.is_compatible(declared_type):
                self.error(f"类型不匹配: 变量 '{node.name}' 声明为 {declared_type}，但初始值为 {value_type}")
            var_type = declared_type
        else:
            var_type = value_type
        
        # 定义变量
        symbol = Symbol(node.name, var_type, is_mutable=node.is_mutable)
        self.current_scope.define(symbol)
    
    def visit_match_statement(self, node: MatchStatement):
        """访问match语句"""
        # 检查匹配表达式类型
        expr_type = self.visit_expression(node.expression)
        
        has_default = False
        
        for item in node.cases:
            if len(item) == 3:
                pattern, guard, body = item
            else:
                pattern, body = item
                guard = None
            
            # 检查是否是默认分支
            is_default = isinstance(pattern, Literal) and pattern.value_type == "DEFAULT"
            is_underscore = isinstance(pattern, Identifier) and pattern.name == "_"
            
            if is_default or is_underscore:
                has_default = True
            else:
                # 检查模式类型
                self.check_pattern(pattern, expr_type)
                
                # 为case体创建新作用域，绑定模式变量
                self.enter_scope()
                self.bind_pattern_variables(pattern, expr_type)
                
                # 检查guard条件
                if guard:
                    guard_type = self.visit_expression(guard)
                    if guard_type.name != "布尔":
                        self.error(f"Guard条件必须是布尔类型，实际为 {guard_type}")
                
                # 分析case体
                for stmt in body:
                    self.visit_statement(stmt)
                
                self.exit_scope()
            
        # 安全方言要求默认分支
        if not has_default and self.dialect == "safe":
            self.error("安全方言要求match语句必须有默认分支 '_'")
    
    def check_pattern(self, pattern: ASTNode, expr_type: Type):
        """检查模式类型"""
        if isinstance(pattern, TuplePattern):
            # 元组模式
            if expr_type.name not in ("元组", "Tuple"):
                self.error(f"元组模式要求表达式为元组类型，实际为 {expr_type}")
        elif isinstance(pattern, RangePattern):
            # 范围模式 - 要求数值类型
            if expr_type.name not in ("整数", "浮点数", "int", "float"):
                self.error(f"范围模式要求数值类型，实际为 {expr_type}")
        # 其他模式类型...
    
    def bind_pattern_variables(self, pattern: ASTNode, expr_type: Type):
        """绑定模式中的变量"""
        if isinstance(pattern, TuplePattern):
            # 元组模式 - 绑定元素变量
            for elem in pattern.elements:
                if isinstance(elem, Identifier) and elem.name not in ('_', '默认'):
                    # 绑定模式变量
                    self.current_scope.define(Symbol(elem.name, Type("Any")))  # TODO: 推断元素类型
        elif isinstance(pattern, Identifier) and pattern.name not in ('_', '默认', 'true', 'false', '真', '假'):
            # 单个模式变量
            self.current_scope.define(Symbol(pattern.name, expr_type))

    def visit_for_loop(self, node: ForLoop):
        """访问for循环"""
        iterable_type = self.visit_expression(node.iterable)
        
        # 进入循环作用域
        self.enter_scope()
        
        # 定义循环变量
        # TODO: 从可迭代类型推断元素类型
        self.current_scope.define(Symbol(node.variable, Type("Any")))
        
        for stmt in node.body:
            self.visit_statement(stmt)
        
        self.exit_scope()
    
    def visit_return_statement(self, node: ReturnStatement):
        """访问return语句"""
        if node.expression:
            return_type = self.visit_expression(node.expression)
            if self.current_function_return_type:
                if not return_type.is_compatible(self.current_function_return_type):
                    self.error(f"返回类型不匹配: 期望 {self.current_function_return_type}，实际 {return_type}")
    
    def visit_expression(self, expr: ASTNode) -> Type:
        """访问表达式并返回类型"""
        if isinstance(expr, Literal):
            return self.visit_literal(expr)
        elif isinstance(expr, Identifier):
            return self.visit_identifier(expr)
        elif isinstance(expr, BinaryExpression):
            return self.visit_binary_expression(expr)
        elif isinstance(expr, UnaryExpression):
            return self.visit_unary_expression(expr)
        elif isinstance(expr, CallExpression):
            return self.visit_call_expression(expr)
        elif isinstance(expr, ArrayLiteral):
            return Type("数组", [Type("Any")])
        elif isinstance(expr, MapLiteral):
            return Type("映射", [Type("Any"), Type("Any")])
        elif isinstance(expr, MemberAccess):
            return Type("Any")  # TODO: 成员类型推断
        elif isinstance(expr, SnapshotExpression):
            return self.visit_snapshot(expr)
        elif isinstance(expr, HandleExpression):
            return self.visit_handle(expr)
        else:
            return Type("Any")
    
    def visit_group_block(self, node: GroupBlock):
        """访问组块"""
        # 检查是否在system方言下
        if self.dialect not in ('system', 'compat'):
            self.error("组块仅在system或compat方言下可用")
            return
        
        group_name = node.name.name if isinstance(node.name, Identifier) else str(node.name)
        
        # 进入组
        self.lifetime_analyzer.enter_group(group_name)
        self.enter_scope()
        
        # 分析组内语句
        for stmt in node.body:
            self.visit_statement(stmt)
        
        # 退出组
        self.exit_scope()
        self.lifetime_analyzer.exit_group()
    
    def visit_snapshot(self, node: SnapshotExpression) -> Type:
        """访问快照表达式"""
        target_type = self.visit_expression(node.target)
        
        # 检查是否在system方言下
        if self.dialect not in ('system', 'compat'):
            self.error("快照仅在system或compat方言下可用")
        
        # 快照是只讻的，不延长源组生命周期
        # 如果目标是标识符，记录其生命周期
        if isinstance(node.target, Identifier):
            var_name = node.target.name
            symbol = self.current_scope.lookup(var_name)
            if symbol:
                symbol.lifetime = Lifetime.SNAPSHOT
        
        return target_type
    
    def visit_handle(self, node: HandleExpression) -> Type:
        """访问句柄表达式"""
        target_type = self.visit_expression(node.target)
        
        # 检查是否在system方言下
        if self.dialect not in ('system', 'compat'):
            self.error("句柄仅在system或compat方言下可用")
        
        # 句柄是轻量引用
        if isinstance(node.target, Identifier):
            var_name = node.target.name
            symbol = self.current_scope.lookup(var_name)
            if symbol:
                symbol.lifetime = Lifetime.HANDLE
        
        return target_type
    
    def visit_literal(self, node: Literal) -> Type:
        """访问字面量"""
        type_map = {
            "INTEGER": Type("整数"),
            "FLOAT": Type("浮点数"),
            "STRING": Type("字符串"),
            "CHAR": Type("字符"),
            "BOOLEAN": Type("布尔"),
            "NIL": Type("单元"),
        }
        return type_map.get(node.value_type, Type("Any"))
    
    def visit_identifier(self, node: Identifier) -> Type:
        """访问标识符"""
        symbol = self.current_scope.lookup(node.name)
        if not symbol:
            self.error(f"未定义的标识符: {node.name}")
            return Type("Any")
        return symbol.type
    
    def visit_binary_expression(self, node: BinaryExpression) -> Type:
        """访问二元表达式"""
        left_type = self.visit_expression(node.left)
        right_type = self.visit_expression(node.right)
        
        # 算术运算
        if node.operator in ("+", "-", "*", "/", "%", "**"):
            if left_type.name in ("整数", "浮点数") and right_type.name in ("整数", "浮点数"):
                if left_type.name == "浮点数" or right_type.name == "浮点数":
                    return Type("浮点数")
                return Type("整数")
            return Type("Any")
        
        # 比较运算
        elif node.operator in ("==", "!=", ">", "<", ">=", "<="):
            return Type("布尔")
        
        # 逻辑运算
        elif node.operator in ("&&", "||", "and", "or"):
            return Type("布尔")
        
        return Type("Any")
    
    def visit_unary_expression(self, node: UnaryExpression) -> Type:
        """访问一元表达式"""
        operand_type = self.visit_expression(node.operand)
        
        if node.operator == "!":
            return Type("布尔")
        elif node.operator == "-":
            return operand_type
        
        return Type("Any")
    
    def visit_call_expression(self, node: CallExpression) -> Type:
        """访问函数调用"""
        # 检查函数是否存在
        if isinstance(node.function, Identifier):
            symbol = self.current_scope.lookup(node.function.name)
            if symbol:
                return symbol.type
        
        # TODO: 更详细的函数签名检查
        return Type("Any")
    
    def resolve_type(self, type_str: str) -> Type:
        """解析类型字符串 - 支持泛型"""
        # 处理泛型类型: 数组<整数>, 映射<字符串, 整数>
        if "<" in type_str:
            # 分解基础类型和类型参数
            base = type_str[:type_str.index("<")].strip()
            params_str = type_str[type_str.index("<")+1:type_str.rindex(">")].strip()
            
            # 处理多个类型参数 (逗号分隔)
            params = []
            depth = 0
            current = []
            for char in params_str:
                if char == '<':
                    depth += 1
                    current.append(char)
                elif char == '>':
                    depth -= 1
                    current.append(char)
                elif char == ',' and depth == 0:
                    params.append(self.resolve_type(''.join(current).strip()))
                    current = []
                else:
                    current.append(char)
            if current:
                params.append(self.resolve_type(''.join(current).strip()))
            
            return Type(base, params)
        
        # 内置类型
        if type_str in BUILTIN_TYPES:
            return BUILTIN_TYPES[type_str]
        
        # 用户定义类型
        return Type(type_str)
    
    def get_errors(self) -> List[str]:
        """获取所有错误"""
        return self.errors
