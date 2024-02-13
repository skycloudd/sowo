use crate::{
    ast::{BinaryOp, Type, UnaryOp},
    mir::{self, Action, ProcId, Statement, VarId},
    FloatTy, IntTy,
};

#[derive(Debug)]
pub struct UnfinishedBasicBlock {
    id: BasicBlockId,
    instructions: Vec<Instruction>,
    terminator: Option<Terminator>,
}

#[derive(Debug)]
pub enum TopLevel {
    Procedure(Procedure),
}

#[derive(Debug)]
pub struct Procedure {
    pub name: ProcId,
    pub args: Vec<(VarId, Type)>,
    pub body: Vec<BasicBlock>,
}

#[derive(Debug)]
pub struct BasicBlock {
    id: BasicBlockId,
    instructions: Vec<Instruction>,
    terminator: Terminator,
}

impl BasicBlock {
    pub const fn id(&self) -> BasicBlockId {
        self.id
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub const fn terminator(&self) -> &Terminator {
        &self.terminator
    }
}

fn finish_block(block: UnfinishedBasicBlock) -> Option<BasicBlock> {
    Some(BasicBlock {
        id: block.id,
        instructions: block.instructions,
        terminator: block.terminator?,
    })
}

#[derive(Debug)]
pub enum Terminator {
    Goto(Goto),
    If {
        condition: TypedExpression,
        then_block: Goto,
        else_block: Goto,
    },
    Call {
        proc: ProcId,
        args: Vec<TypedExpression>,
        continuation: Goto,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Goto {
    Block(BasicBlockId),
    Return,
}

pub type BasicBlockId = usize;

#[derive(Debug)]
pub enum Instruction {
    Expr(TypedExpression),
    Let {
        name: VarId,
        value: TypedExpression,
    },
    Assign {
        name: VarId,
        value: TypedExpression,
    },
    Action {
        name: Action,
        args: Vec<TypedExpression>,
    },
}

#[derive(Debug)]
pub struct TypedExpression {
    pub expr: Expression,
    pub ty: Type,
}

#[derive(Debug)]
pub enum Expression {
    Variable(VarId),
    Boolean(bool),
    Integer(IntTy),
    Float(FloatTy),
    Colour {
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    },
    Vector {
        x: Box<TypedExpression>,
        y: Box<TypedExpression>,
    },
    Unary {
        op: UnaryOp,
        value: Box<TypedExpression>,
    },
    Binary {
        lhs: Box<TypedExpression>,
        op: BinaryOp,
        rhs: Box<TypedExpression>,
    },
}

pub fn lower(ast: Vec<mir::TopLevel>) -> Vec<TopLevel> {
    LoweringContext::default().lower(ast)
}

#[derive(Debug, Default)]
struct LoweringContext {
    blocks: Vec<UnfinishedBasicBlock>,
    current_block: BasicBlockId,
    loop_stack: Vec<(BasicBlockId, BasicBlockId)>,
}

impl LoweringContext {
    fn new_block(&mut self) -> BasicBlockId {
        let id = self.blocks.len();

        self.blocks.push(UnfinishedBasicBlock {
            id,
            instructions: vec![],
            terminator: None,
        });

        id
    }

    fn switch_to(&mut self, block: BasicBlockId) {
        self.current_block = block;
    }

    fn current(&self) -> &UnfinishedBasicBlock {
        self.blocks.get(self.current_block).unwrap()
    }

    fn current_mut(&mut self) -> &mut UnfinishedBasicBlock {
        self.blocks.get_mut(self.current_block).unwrap()
    }

    fn finish(&mut self, terminator: Terminator) {
        assert!(
            self.current().terminator.is_none(),
            "block {} already finished",
            self.current_block
        );

        self.current_mut().terminator = Some(terminator);
    }

    fn finish_checked(&mut self, terminator: Terminator) {
        if self.current().terminator.is_none() {
            self.current_mut().terminator = Some(terminator);
        }
    }

    fn loop_stack_push(&mut self, loop_: BasicBlockId, merge: BasicBlockId) {
        self.loop_stack.push((loop_, merge));
    }

    fn loop_stack_pop(&mut self) -> (BasicBlockId, BasicBlockId) {
        self.loop_stack.pop().unwrap()
    }

    fn loop_stack_top(&self) -> (BasicBlockId, BasicBlockId) {
        *self.loop_stack.last().unwrap()
    }

    fn lower(mut self, ast: Vec<mir::TopLevel>) -> Vec<TopLevel> {
        ast.into_iter()
            .map(|top_level| match top_level {
                mir::TopLevel::Procedure(procedure) => {
                    self.lower_statements(procedure.body);

                    let body = self.blocks.drain(..).filter_map(finish_block).collect();

                    TopLevel::Procedure(Procedure {
                        name: procedure.name,
                        args: procedure.args,
                        body,
                    })
                }
            })
            .collect()
    }

    fn lower_statements(&mut self, ast: Vec<Statement>) {
        self.blocks.clear();

        let start = self.new_block();
        self.switch_to(start);

        for statement in ast {
            self.lower_statement(statement);
        }

        self.finish(Terminator::Goto(Goto::Return));
    }

    fn lower_statement(&mut self, statement: Statement) -> bool {
        match statement {
            Statement::Expr(expr) => {
                let expr = Self::lower_expression(expr);

                self.current_mut()
                    .instructions
                    .push(Instruction::Expr(expr));
            }
            Statement::Block(statements) => {
                for statement in statements {
                    if self.lower_statement(statement) {
                        break;
                    }
                }
            }
            Statement::Loop(statements) => {
                let loop_start = self.new_block();

                let merge_block = self.new_block();

                self.finish(Terminator::Goto(Goto::Block(loop_start)));

                self.switch_to(loop_start);

                self.loop_stack_push(loop_start, merge_block);

                for statement in statements {
                    if self.lower_statement(statement) {
                        break;
                    }
                }

                self.loop_stack_pop();

                self.finish_checked(Terminator::Goto(Goto::Block(loop_start)));

                self.switch_to(merge_block);
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = Self::lower_expression(condition);

                let then_block = self.new_block();

                let else_block = else_branch.as_ref().map(|_| self.new_block());

                let merge_block = self.new_block();

                self.finish(Terminator::If {
                    condition,
                    then_block: Goto::Block(then_block),
                    else_block: Goto::Block(else_block.unwrap_or(merge_block)),
                });

                self.switch_to(then_block);

                for statement in then_branch {
                    if self.lower_statement(statement) {
                        break;
                    }
                }

                self.finish_checked(Terminator::Goto(Goto::Block(merge_block)));

                if let Some(else_block) = else_block {
                    self.switch_to(else_block);

                    if let Some(else_branch) = else_branch {
                        for statement in else_branch {
                            if self.lower_statement(statement) {
                                break;
                            }
                        }
                    }

                    self.finish_checked(Terminator::Goto(Goto::Block(merge_block)));
                }

                self.switch_to(merge_block);
            }
            Statement::Let { name, value } => {
                let value = Self::lower_expression(value);

                self.current_mut()
                    .instructions
                    .push(Instruction::Let { name, value });
            }
            Statement::Assign { name, value } => {
                let value = Self::lower_expression(value);

                self.current_mut()
                    .instructions
                    .push(Instruction::Assign { name, value });
            }
            Statement::Break => {
                let merge_block = self.loop_stack_top().1;

                self.finish(Terminator::Goto(Goto::Block(merge_block)));

                return true;
            }
            Statement::Continue => {
                let loop_block = self.loop_stack_top().0;

                self.finish(Terminator::Goto(Goto::Block(loop_block)));

                return true;
            }
            Statement::Return => {
                self.finish(Terminator::Goto(Goto::Return));

                return true;
            }
            Statement::Action { action: name, args } => {
                let args = args.into_iter().map(Self::lower_expression).collect();

                self.current_mut()
                    .instructions
                    .push(Instruction::Action { name, args });
            }
            Statement::Call { proc, args } => {
                let args = args.into_iter().map(Self::lower_expression).collect();

                let continuation = self.new_block();

                self.finish(Terminator::Call {
                    proc,
                    args,
                    continuation: Goto::Block(continuation),
                });

                self.switch_to(continuation);
            }
        }

        false
    }

    fn lower_expression(expression: mir::TypedExpression) -> TypedExpression {
        TypedExpression {
            expr: match expression.expr {
                mir::Expression::Variable(name) => Expression::Variable(name),
                mir::Expression::Boolean(value) => Expression::Boolean(value),
                mir::Expression::Integer(value) => Expression::Integer(value),
                mir::Expression::Float(value) => Expression::Float(value),
                mir::Expression::Colour { r, g, b, a } => Expression::Colour { r, g, b, a },
                mir::Expression::Vector { x, y } => Expression::Vector {
                    x: Box::new(Self::lower_expression(*x)),
                    y: Box::new(Self::lower_expression(*y)),
                },
                mir::Expression::Unary { op, rhs } => Expression::Unary {
                    op,
                    value: Box::new(Self::lower_expression(*rhs)),
                },
                mir::Expression::Binary { lhs, op, rhs } => Expression::Binary {
                    lhs: Box::new(Self::lower_expression(*lhs)),
                    op,
                    rhs: Box::new(Self::lower_expression(*rhs)),
                },
            },
            ty: expression.ty,
        }
    }
}

mod print {
    use super::{BasicBlock, Expression, Goto, Instruction, Procedure, Terminator, TopLevel};
    use crate::mir::Action;

    impl std::fmt::Display for TopLevel {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            print_toplevel(f, self)
        }
    }

    fn print_toplevel(f: &mut std::fmt::Formatter<'_>, top_level: &TopLevel) -> std::fmt::Result {
        match top_level {
            TopLevel::Procedure(procedure) => print_procedure(f, procedure),
        }
    }

    fn print_procedure(f: &mut std::fmt::Formatter<'_>, procedure: &Procedure) -> std::fmt::Result {
        write!(f, "proc proc_{}(", procedure.name.0)?;

        for (i, arg) in procedure.args.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            write!(f, "var_{}: {}", arg.0 .0, arg.1)?;
        }

        writeln!(f, "):")?;

        for block in &procedure.body {
            print_basic_block(f, block)?;
        }

        Ok(())
    }

    fn print_basic_block(f: &mut std::fmt::Formatter<'_>, block: &BasicBlock) -> std::fmt::Result {
        writeln!(f, "    bb{}:", block.id)?;

        for instruction in &block.instructions {
            write!(f, "        ")?;

            print_instruction(f, instruction)?;

            writeln!(f)?;
        }

        write!(f, "        ")?;
        print_terminator(f, &block.terminator)?;

        writeln!(f)
    }

    fn print_instruction(
        f: &mut std::fmt::Formatter<'_>,
        instruction: &Instruction,
    ) -> std::fmt::Result {
        match instruction {
            Instruction::Expr(expr) => {
                print_expression(f, &expr.expr)?;

                write!(f, ";")
            }
            Instruction::Let { name, value } => {
                write!(f, "let var_{} = ", name.0)?;

                print_expression(f, &value.expr)?;

                write!(f, ";")
            }
            Instruction::Assign { name, value } => {
                write!(f, "var_{} = ", name.0)?;

                print_expression(f, &value.expr)?;

                write!(f, ";")
            }
            Instruction::Action { name, args } => {
                write!(f, "action {name}: ")?;

                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    print_expression(f, &arg.expr)?;
                }

                write!(f, ";")
            }
        }
    }

    fn print_terminator(
        f: &mut std::fmt::Formatter<'_>,
        terminator: &Terminator,
    ) -> std::fmt::Result {
        match terminator {
            Terminator::Goto(goto) => write!(f, "goto {goto};"),
            Terminator::If {
                condition,
                then_block,
                else_block,
            } => {
                write!(f, "if ")?;

                print_expression(f, &condition.expr)?;

                write!(f, " then {then_block} else {else_block};")
            }
            Terminator::Call {
                proc,
                args,
                continuation,
            } => {
                write!(f, "call proc_{}(", proc.0)?;

                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    print_expression(f, &arg.expr)?;
                }

                write!(f, ") then {continuation};")
            }
        }
    }

    fn print_expression(
        f: &mut std::fmt::Formatter<'_>,
        expression: &Expression,
    ) -> std::fmt::Result {
        write!(f, "(")?;

        match expression {
            Expression::Variable(name) => write!(f, "var_{}", name.0)?,
            Expression::Boolean(value) => write!(f, "{value}")?,
            Expression::Integer(value) => write!(f, "{value}")?,
            Expression::Float(value) => write!(f, "{value}")?,

            Expression::Colour { r, g, b, a } => write!(f, "#{r:02x}{g:02x}{b:02x}{a:02x}")?,
            Expression::Vector { x, y } => {
                write!(f, "{{ ")?;

                print_expression(f, &x.expr)?;

                write!(f, ", ")?;

                print_expression(f, &y.expr)?;

                write!(f, " }}")?;
            }
            Expression::Unary { op, value } => {
                write!(f, "{op}")?;

                print_expression(f, &value.expr)?;
            }
            Expression::Binary { lhs, op, rhs } => {
                print_expression(f, &lhs.expr)?;

                write!(f, " {op} ")?;

                print_expression(f, &rhs.expr)?;
            }
        }

        write!(f, ")")
    }

    impl std::fmt::Display for Goto {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Block(block) => write!(f, "bb{block}"),
                Self::Return => write!(f, "return"),
            }
        }
    }

    impl std::fmt::Display for Action {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Wait => write!(f, "wait"),
                Self::WaitFrames => write!(f, "waitframes"),
                Self::Print => write!(f, "print"),
            }
        }
    }
}
